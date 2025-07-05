use chia::{
    clvm_traits::{FromClvm, ToClvm},
    protocol::{Bytes32, Coin, Program},
    puzzles::{nft::NftMetadata, LineageProof, Memos, Proof},
};
use chia_puzzles::SINGLETON_LAUNCHER_HASH;
use chia_wallet_sdk::{
    driver::{Cat, CatInfo, ClawbackV2, Did, DidInfo, HashedPtr, Nft, NftInfo, Puzzle},
    prelude::CreateCoin,
    types::{run_puzzle, Condition},
};
use clvmr::{Allocator, NodePtr};
use tracing::{debug_span, warn};

use crate::WalletError;

#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum ChildKind {
    Unknown,
    Launcher,
    Clawback {
        info: ClawbackV2,
    },
    Cat {
        info: CatInfo,
        lineage_proof: LineageProof,
        clawback: Option<ClawbackV2>,
    },
    Did {
        info: DidInfo<Program>,
        lineage_proof: LineageProof,
        clawback: Option<ClawbackV2>,
    },
    Nft {
        info: NftInfo<Program>,
        lineage_proof: LineageProof,
        metadata: Option<NftMetadata>,
        clawback: Option<ClawbackV2>,
    },
}

impl ChildKind {
    pub fn from_parent(
        parent_coin: Coin,
        parent_puzzle: &Program,
        parent_solution: &Program,
        coin: Coin,
    ) -> Result<Self, WalletError> {
        let mut allocator = Allocator::new();

        let parent_puzzle_ptr = parent_puzzle.to_clvm(&mut allocator)?;
        let parent_puzzle = Puzzle::parse(&allocator, parent_puzzle_ptr);
        let parent_solution = parent_solution.to_clvm(&mut allocator)?;

        Self::from_parent_cached(
            &mut allocator,
            parent_coin,
            parent_puzzle,
            parent_solution,
            coin,
        )
    }

    pub fn from_parent_cached(
        allocator: &mut Allocator,
        parent_coin: Coin,
        parent_puzzle: Puzzle,
        parent_solution: NodePtr,
        coin: Coin,
    ) -> Result<Self, WalletError> {
        let parse_span = debug_span!(
            "parse from parent",
            parent_coin = %parent_coin.coin_id(),
            coin = %coin.coin_id()
        );
        let _span = parse_span.enter();

        if coin.puzzle_hash == SINGLETON_LAUNCHER_HASH.into() {
            return Ok(Self::Launcher);
        }

        let output = run_puzzle(allocator, parent_puzzle.ptr(), parent_solution)?;
        let conditions = Vec::<Condition>::from_clvm(allocator, output)?;

        let Some(create_coin) = conditions
            .into_iter()
            .filter_map(Condition::into_create_coin)
            .find(|create_coin| {
                let child_coin = Coin::new(
                    parent_coin.coin_id(),
                    create_coin.puzzle_hash,
                    create_coin.amount,
                );

                child_coin == coin
            })
        else {
            return Ok(Self::Unknown);
        };

        match Cat::parse_children(allocator, parent_coin, parent_puzzle, parent_solution) {
            // If there was an error parsing the CAT, we can exit early.
            Err(error) => {
                warn!("Invalid CAT: {}", error);
                return Ok(Self::Unknown);
            }

            // If the coin is a CAT coin, return the relevant information.
            Ok(Some(cats)) => {
                let Some(cat) = cats.into_iter().find(|cat| cat.coin == coin) else {
                    warn!("CAT coin not found in children");
                    return Ok(Self::Unknown);
                };

                // We don't support parsing eve CATs during syncing.
                let Some(lineage_proof) = cat.lineage_proof else {
                    return Ok(Self::Unknown);
                };

                let clawback =
                    parse_clawback(allocator, &create_coin, true, cat.info.p2_puzzle_hash);

                return Ok(Self::Cat {
                    info: cat.info,
                    lineage_proof,
                    clawback,
                });
            }

            // If the coin is not a CAT coin, continue parsing.
            Ok(None) => {}
        }

        match Nft::<HashedPtr>::parse_child(allocator, parent_coin, parent_puzzle, parent_solution)
        {
            // If there was an error parsing the NFT, we can exit early.
            Err(error) => {
                warn!("Invalid NFT: {}", error);
                return Ok(Self::Unknown);
            }

            // If the coin is a NFT coin, return the relevant information.
            Ok(Some(nft)) => {
                if nft.coin != coin {
                    warn!(
                        "NFT coin {:?} does not match expected coin {:?}",
                        nft.coin, coin
                    );
                    return Ok(Self::Unknown);
                }

                // We don't support parsing eve NFTs during syncing.
                let Proof::Lineage(lineage_proof) = nft.proof else {
                    return Ok(Self::Unknown);
                };

                let metadata_program = Program::from_clvm(allocator, nft.info.metadata.ptr())?;
                let metadata = NftMetadata::from_clvm(allocator, nft.info.metadata.ptr()).ok();

                let clawback =
                    parse_clawback(allocator, &create_coin, true, nft.info.p2_puzzle_hash);

                return Ok(Self::Nft {
                    lineage_proof,
                    info: nft.info.with_metadata(metadata_program),
                    metadata,
                    clawback,
                });
            }

            // If the coin is not a NFT coin, continue parsing.
            Ok(None) => {}
        }

        match Did::<HashedPtr>::parse_child(
            allocator,
            parent_coin,
            parent_puzzle,
            parent_solution,
            coin,
        ) {
            // If there was an error parsing the DID, we can exit early.
            Err(error) => {
                warn!("Invalid DID: {}", error);
                return Ok(Self::Unknown);
            }

            // If the coin is a DID coin, return the relevant information.
            Ok(Some(did)) => {
                // We don't support parsing eve DIDs during syncing.
                let Proof::Lineage(lineage_proof) = did.proof else {
                    return Ok(Self::Unknown);
                };

                let metadata = Program::from_clvm(allocator, did.info.metadata.ptr())?;

                let clawback =
                    parse_clawback(allocator, &create_coin, true, did.info.p2_puzzle_hash);

                return Ok(Self::Did {
                    lineage_proof,
                    info: did.info.with_metadata(metadata),
                    clawback,
                });
            }

            // If the coin is not a DID coin, continue parsing.
            Ok(None) => {}
        }

        if let Some(clawback) = parse_clawback(allocator, &create_coin, false, coin.puzzle_hash) {
            return Ok(Self::Clawback { info: clawback });
        }

        Ok(Self::Unknown)
    }

    pub fn custody_p2_puzzle_hash(&self) -> Option<Bytes32> {
        match self {
            Self::Launcher | Self::Unknown => None,
            Self::Clawback { info } => Some(info.receiver_puzzle_hash),
            Self::Cat { info, clawback, .. } => {
                Some(clawback.map_or(info.p2_puzzle_hash, |clawback| {
                    clawback.receiver_puzzle_hash
                }))
            }
            Self::Did { info, clawback, .. } => {
                Some(clawback.map_or(info.p2_puzzle_hash, |clawback| {
                    clawback.receiver_puzzle_hash
                }))
            }
            Self::Nft { info, clawback, .. } => {
                Some(clawback.map_or(info.p2_puzzle_hash, |clawback| {
                    clawback.receiver_puzzle_hash
                }))
            }
        }
    }

    pub fn subscribe(&self) -> bool {
        matches!(
            self,
            Self::Clawback { .. } | Self::Cat { .. } | Self::Did { .. } | Self::Nft { .. }
        )
    }
}

fn parse_clawback(
    allocator: &Allocator,
    create_coin: &CreateCoin<NodePtr>,
    hinted: bool,
    p2_puzzle_hash: Bytes32,
) -> Option<ClawbackV2> {
    let Memos::Some(memos) = create_coin.memos else {
        return None;
    };

    let (hint, (clawback_memo, _)) =
        <(Bytes32, (NodePtr, NodePtr))>::from_clvm(allocator, memos).ok()?;

    ClawbackV2::from_memo(
        allocator,
        clawback_memo,
        hint,
        create_coin.amount,
        hinted,
        p2_puzzle_hash,
    )
}

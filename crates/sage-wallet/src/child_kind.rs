use chia::{
    clvm_traits::{FromClvm, ToClvm},
    protocol::{Bytes32, Coin, Program},
    puzzles::{nft::NftMetadata, LineageProof, Proof},
};
use chia_puzzles::SINGLETON_LAUNCHER_HASH;
use chia_wallet_sdk::{
    driver::{Cat, Did, DidInfo, HashedPtr, Nft, NftInfo, Puzzle},
    prelude::Memos,
    types::{run_puzzle, Condition},
};
use clvmr::{Allocator, NodePtr};
use tracing::{debug_span, warn};

use crate::WalletError;

#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum ChildKind {
    Unknown {
        hint: Option<Bytes32>,
    },
    Launcher,
    Cat {
        asset_id: Bytes32,
        p2_puzzle_hash: Bytes32,
        lineage_proof: LineageProof,
    },
    Did {
        info: DidInfo<Program>,
        lineage_proof: LineageProof,
    },
    Nft {
        info: NftInfo<Program>,
        lineage_proof: LineageProof,
        metadata: Option<NftMetadata>,
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

        let output = run_puzzle(&mut allocator, parent_puzzle.ptr(), parent_solution)?;
        let conditions = Vec::<Condition>::from_clvm(&allocator, output)?;

        Self::from_parent_cached(
            &mut allocator,
            parent_coin,
            parent_puzzle,
            parent_solution,
            conditions,
            coin,
        )
    }

    pub fn from_parent_cached(
        allocator: &mut Allocator,
        parent_coin: Coin,
        parent_puzzle: Puzzle,
        parent_solution: NodePtr,
        conditions: Vec<Condition>,
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

        let Some(create_coin) = conditions
            .into_iter()
            .filter_map(Condition::into_create_coin)
            .find(|cond| cond.puzzle_hash == coin.puzzle_hash && cond.amount == coin.amount)
        else {
            return Ok(Self::Unknown { hint: None });
        };

        let hint = if let Some(memos) = create_coin.memos {
            let memos = Memos::<(Bytes32, NodePtr)>::from_clvm(allocator, memos.value).ok();
            memos.map(|memos| memos.value.0)
        } else {
            None
        };

        let unknown = Self::Unknown { hint };

        match Cat::parse_children(allocator, parent_coin, parent_puzzle, parent_solution) {
            // If there was an error parsing the CAT, we can exit early.
            Err(error) => {
                warn!("Invalid CAT: {}", error);
                return Ok(unknown);
            }

            // If the coin is a CAT coin, return the relevant information.
            Ok(Some(cats)) => {
                let Some(cat) = cats.into_iter().find(|cat| cat.coin == coin) else {
                    warn!("CAT coin not found in children");
                    return Ok(unknown);
                };

                // We don't support parsing eve CATs during syncing.
                let Some(lineage_proof) = cat.lineage_proof else {
                    return Ok(unknown);
                };

                return Ok(Self::Cat {
                    asset_id: cat.asset_id,
                    lineage_proof,
                    p2_puzzle_hash: cat.p2_puzzle_hash,
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
                return Ok(unknown);
            }

            // If the coin is a NFT coin, return the relevant information.
            Ok(Some(nft)) => {
                if nft.coin != coin {
                    warn!("NFT coin does not match expected coin");
                    return Ok(unknown);
                }

                // We don't support parsing eve NFTs during syncing.
                let Proof::Lineage(lineage_proof) = nft.proof else {
                    return Ok(unknown);
                };

                let metadata_program = Program::from_clvm(allocator, nft.info.metadata.ptr())?;
                let metadata = NftMetadata::from_clvm(allocator, nft.info.metadata.ptr()).ok();

                return Ok(Self::Nft {
                    lineage_proof,
                    info: nft.info.with_metadata(metadata_program),
                    metadata,
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
                return Ok(unknown);
            }

            // If the coin is a DID coin, return the relevant information.
            Ok(Some(did)) => {
                // We don't support parsing eve DIDs during syncing.
                let Proof::Lineage(lineage_proof) = did.proof else {
                    return Ok(unknown);
                };

                let metadata = Program::from_clvm(allocator, did.info.metadata.ptr())?;

                return Ok(Self::Did {
                    lineage_proof,
                    info: did.info.with_metadata(metadata),
                });
            }

            // If the coin is not a DID coin, continue parsing.
            Ok(None) => {}
        }

        Ok(unknown)
    }

    pub fn p2_puzzle_hash(&self) -> Option<Bytes32> {
        match self {
            Self::Launcher | Self::Unknown { .. } => None,
            Self::Cat { p2_puzzle_hash, .. } => Some(*p2_puzzle_hash),
            Self::Did { info, .. } => Some(info.p2_puzzle_hash),
            Self::Nft { info, .. } => Some(info.p2_puzzle_hash),
        }
    }

    pub fn subscribe(&self) -> bool {
        matches!(self, Self::Cat { .. } | Self::Did { .. } | Self::Nft { .. })
    }
}

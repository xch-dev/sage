use chia::{
    clvm_traits::{FromClvm, ToClvm},
    clvm_utils::ToTreeHash,
    protocol::{Bytes32, Coin, Program},
    puzzles::{nft::NftMetadata, LineageProof, Memos, Proof},
};
use chia_puzzles::SINGLETON_LAUNCHER_HASH;
use chia_wallet_sdk::{
    driver::{Cat, CatInfo, ClawbackV2, Did, DidInfo, Nft, Puzzle, SingletonInfo},
    prelude::CreateCoin,
    types::{run_puzzle, Condition},
};
use clvmr::{Allocator, NodePtr};
use sage_database::{SerializePrimitive, SerializedDidInfo, SerializedNftInfo};
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
        info: SerializedDidInfo,
        lineage_proof: LineageProof,
        clawback: Option<ClawbackV2>,
    },
    Nft {
        info: SerializedNftInfo,
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

        let output = run_puzzle(&mut allocator, parent_puzzle.ptr(), parent_solution)?;
        let conditions = Vec::<Condition>::from_clvm(&allocator, output)?;

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

        Self::from_parent_cached(
            &mut allocator,
            parent_coin,
            parent_puzzle,
            parent_solution,
            create_coin,
        )
    }

    pub fn parse_children(
        parent_coin: Coin,
        parent_puzzle: &Program,
        parent_solution: &Program,
    ) -> Result<Vec<(Coin, Self)>, WalletError> {
        let mut allocator = Allocator::new();

        let parent_puzzle_ptr = parent_puzzle.to_clvm(&mut allocator)?;
        let parent_puzzle = Puzzle::parse(&allocator, parent_puzzle_ptr);
        let parent_solution = parent_solution.to_clvm(&mut allocator)?;

        let output = run_puzzle(&mut allocator, parent_puzzle.ptr(), parent_solution)?;
        let conditions = Vec::<Condition>::from_clvm(&allocator, output)?;

        Ok(conditions
            .into_iter()
            .filter_map(Condition::into_create_coin)
            .filter_map(|create_coin| {
                Self::from_parent_cached(
                    &mut allocator,
                    parent_coin,
                    parent_puzzle,
                    parent_solution,
                    create_coin,
                )
                .ok()
                .map(|kind| {
                    (
                        Coin::new(
                            parent_coin.coin_id(),
                            create_coin.puzzle_hash,
                            create_coin.amount,
                        ),
                        kind,
                    )
                })
            })
            .collect())
    }

    fn from_parent_cached(
        allocator: &mut Allocator,
        parent_coin: Coin,
        parent_puzzle: Puzzle,
        parent_solution: NodePtr,
        create_coin: CreateCoin<NodePtr>,
    ) -> Result<Self, WalletError> {
        let coin = Coin::new(
            parent_coin.coin_id(),
            create_coin.puzzle_hash,
            create_coin.amount,
        );

        let parse_span = debug_span!(
            "parse from parent",
            parent_coin = %parent_coin.coin_id(),
            coin = %coin.coin_id()
        );
        let _span = parse_span.enter();

        if coin.puzzle_hash == SINGLETON_LAUNCHER_HASH.into() {
            return Ok(Self::Launcher);
        }

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

                let clawback = parse_clawback_unchecked(allocator, &create_coin, true)
                    .filter(|clawback| clawback.tree_hash() == cat.info.p2_puzzle_hash.into());

                return Ok(Self::Cat {
                    info: cat.info,
                    lineage_proof,
                    clawback,
                });
            }

            // If the coin is not a CAT coin, continue parsing.
            Ok(None) => {}
        }

        if coin.amount % 2 == 1 {
            match Nft::parse_child(allocator, parent_coin, parent_puzzle, parent_solution) {
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

                    let metadata = NftMetadata::from_clvm(allocator, nft.info.metadata.ptr()).ok();

                    let clawback = parse_clawback_unchecked(allocator, &create_coin, true)
                        .filter(|clawback| clawback.tree_hash() == nft.info.p2_puzzle_hash.into());

                    return Ok(Self::Nft {
                        lineage_proof,
                        info: nft.serialize(allocator)?.info,
                        metadata,
                        clawback,
                    });
                }

                // If the coin is not a NFT coin, continue parsing.
                Ok(None) => {}
            }

            match Did::parse_child(allocator, parent_coin, parent_puzzle, parent_solution, coin) {
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

                    let clawback = parse_clawback_unchecked(allocator, &create_coin, true);

                    let did = if let Some(clawback) = clawback {
                        let p2_puzzle_hash = clawback.tree_hash().into();

                        let clawback_did = Did::new(
                            did.coin,
                            did.proof,
                            DidInfo::new(
                                did.info.launcher_id,
                                did.info.recovery_list_hash,
                                did.info.num_verifications_required,
                                did.info.metadata,
                                p2_puzzle_hash,
                            ),
                        );

                        if clawback_did.info.puzzle_hash() == coin.puzzle_hash.into() {
                            clawback_did
                        } else {
                            did
                        }
                    } else {
                        did
                    };

                    return Ok(Self::Did {
                        lineage_proof,
                        info: did.serialize(allocator)?.info,
                        clawback,
                    });
                }

                // If the coin is not a DID coin, continue parsing.
                Ok(None) => {}
            }
        }

        if let Some(clawback) = parse_clawback_unchecked(allocator, &create_coin, false)
            .filter(|clawback| clawback.tree_hash() == coin.puzzle_hash.into())
        {
            return Ok(Self::Clawback { info: clawback });
        }

        Ok(Self::Unknown)
    }

    pub fn custody_p2_puzzle_hashes(&self) -> Vec<Bytes32> {
        match self {
            // TODO: Should we add the puzzle hash of the coin as a candidate?
            Self::Launcher | Self::Unknown => vec![],
            Self::Clawback { info } => vec![info.sender_puzzle_hash, info.receiver_puzzle_hash],
            Self::Cat { info, clawback, .. } => clawback
                .map_or(vec![info.p2_puzzle_hash], |clawback| {
                    vec![clawback.sender_puzzle_hash, clawback.receiver_puzzle_hash]
                }),
            Self::Did { info, clawback, .. } => clawback
                .map_or(vec![info.p2_puzzle_hash], |clawback| {
                    vec![clawback.sender_puzzle_hash, clawback.receiver_puzzle_hash]
                }),
            Self::Nft { info, clawback, .. } => clawback
                .map_or(vec![info.p2_puzzle_hash], |clawback| {
                    vec![clawback.sender_puzzle_hash, clawback.receiver_puzzle_hash]
                }),
        }
    }

    pub fn receiver_custody_p2_puzzle_hash(&self) -> Option<Bytes32> {
        match self {
            Self::Launcher | Self::Unknown => None,
            Self::Clawback { info } => Some(info.receiver_puzzle_hash),
            Self::Cat { info, clawback, .. } => clawback
                .map_or(Some(info.p2_puzzle_hash), |clawback| {
                    Some(clawback.receiver_puzzle_hash)
                }),
            Self::Did { info, clawback, .. } => clawback
                .map_or(Some(info.p2_puzzle_hash), |clawback| {
                    Some(clawback.receiver_puzzle_hash)
                }),
            Self::Nft { info, clawback, .. } => clawback
                .map_or(Some(info.p2_puzzle_hash), |clawback| {
                    Some(clawback.receiver_puzzle_hash)
                }),
        }
    }

    pub fn subscribe(&self) -> bool {
        matches!(
            self,
            Self::Clawback { .. } | Self::Cat { .. } | Self::Did { .. } | Self::Nft { .. }
        )
    }
}

fn parse_clawback_unchecked(
    allocator: &Allocator,
    create_coin: &CreateCoin<NodePtr>,
    hinted: bool,
) -> Option<ClawbackV2> {
    let Memos::Some(memos) = create_coin.memos else {
        return None;
    };

    let (hint, (clawback_memo, _)) =
        <(Bytes32, (NodePtr, NodePtr))>::from_clvm(allocator, memos).ok()?;

    clawback_from_memo_unchecked(allocator, clawback_memo, hint, create_coin.amount, hinted)
}

pub fn clawback_from_memo_unchecked(
    allocator: &Allocator,
    memo: NodePtr,
    receiver_puzzle_hash: Bytes32,
    amount: u64,
    hinted: bool,
) -> Option<ClawbackV2> {
    let (sender_puzzle_hash, (seconds, ())) =
        <(Bytes32, (u64, ()))>::from_clvm(allocator, memo).ok()?;

    let clawback = ClawbackV2 {
        sender_puzzle_hash,
        receiver_puzzle_hash,
        seconds,
        amount,
        hinted,
    };

    Some(clawback)
}

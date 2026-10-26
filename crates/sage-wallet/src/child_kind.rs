use chia_wallet_sdk::{
    chia::puzzle_types::{LineageProof, Proof, nft::NftMetadata},
    driver::{Clawback as ClawbackV1, ClawbackV2},
    prelude::*,
    puzzles::SINGLETON_LAUNCHER_HASH,
};
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
    ClawbackV1 {
        info: ClawbackV1,
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
    Option {
        info: OptionInfo,
        lineage_proof: LineageProof,
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
            &mut None,
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

        let mut clawbacks_v1 = None;

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
                    &mut clawbacks_v1,
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
        clawbacks_v1: &mut Option<Vec<ClawbackV1>>,
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

                    // The clawback memos are only trusted if the DID's puzzle actually
                    // commits to the clawback as its owner.
                    let (did, clawback) = if let Some(clawback) = clawback {
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
                            (clawback_did, Some(clawback))
                        } else {
                            (did, None)
                        }
                    } else {
                        (did, None)
                    };

                    // The p2 puzzle hash comes from the hint, which anyone can choose, so it
                    // must produce the coin that was actually created.
                    if did.info.puzzle_hash() != coin.puzzle_hash.into() {
                        warn!(
                            "DID coin {:?} does not match the puzzle hash implied by its hint",
                            coin.coin_id()
                        );
                        return Ok(Self::Unknown);
                    }

                    return Ok(Self::Did {
                        lineage_proof,
                        info: did.serialize(allocator)?.info,
                        clawback,
                    });
                }

                // If the coin is not a DID coin, continue parsing.
                Ok(None) => {}
            }

            match OptionContract::parse_child(
                allocator,
                parent_coin,
                parent_puzzle,
                parent_solution,
            ) {
                // If there was an error parsing the option contract, we can exit early.
                Err(error) => {
                    warn!("Invalid option contract: {}", error);
                    return Ok(Self::Unknown);
                }

                // If the coin is an option contract, return the relevant information.
                Ok(Some(option)) => {
                    if option.coin != coin {
                        warn!(
                            "Option contract coin {:?} does not match expected coin {:?}",
                            option.coin, coin
                        );
                        return Ok(Self::Unknown);
                    }

                    // We don't support parsing eve option contracts during syncing.
                    let Proof::Lineage(lineage_proof) = option.proof else {
                        return Ok(Self::Unknown);
                    };

                    let clawback = parse_clawback_unchecked(allocator, &create_coin, true).filter(
                        |clawback| clawback.tree_hash() == option.info.p2_puzzle_hash.into(),
                    );

                    return Ok(Self::Option {
                        lineage_proof,
                        info: option.info,
                        clawback,
                    });
                }

                // If the coin is not an option contract, continue parsing.
                Ok(None) => {}
            }
        }

        if let Some(clawback) = parse_clawback_unchecked(allocator, &create_coin, false)
            .filter(|clawback| clawback.tree_hash() == coin.puzzle_hash.into())
        {
            return Ok(Self::Clawback { info: clawback });
        }

        // V1 structure lives in parent REMARK conditions (not CREATE_COIN memos). Parsing them
        // requires running the parent puzzle, so do it at most once for all of its children.
        let clawbacks_v1 = clawbacks_v1.get_or_insert_with(|| {
            ClawbackV1::parse_children(allocator, parent_puzzle, parent_solution)
                .unwrap_or_else(|error| {
                    warn!("Failed to parse clawback v1 from parent: {error}");
                    None
                })
                .unwrap_or_default()
                .into_iter()
                .filter(|clawback| i64::try_from(clawback.timelock).is_ok())
                .collect()
        });

        if let Some(&info) = clawbacks_v1
            .iter()
            .find(|clawback| clawback.tree_hash() == coin.puzzle_hash.into())
        {
            return Ok(Self::ClawbackV1 { info });
        }

        Ok(Self::Unknown)
    }

    pub fn custody_p2_puzzle_hashes(&self) -> Vec<Bytes32> {
        match self {
            // TODO: Should we add the puzzle hash of the coin as a candidate?
            Self::Launcher | Self::Unknown => vec![],
            Self::Clawback { info } => vec![info.sender_puzzle_hash, info.receiver_puzzle_hash],
            Self::ClawbackV1 { info } => vec![info.sender_puzzle_hash, info.receiver_puzzle_hash],
            Self::Cat {
                info: CatInfo { p2_puzzle_hash, .. },
                clawback,
                ..
            }
            | Self::Did {
                info: SerializedDidInfo { p2_puzzle_hash, .. },
                clawback,
                ..
            }
            | Self::Nft {
                info: SerializedNftInfo { p2_puzzle_hash, .. },
                clawback,
                ..
            }
            | Self::Option {
                info: OptionInfo { p2_puzzle_hash, .. },
                clawback,
                ..
            } => clawback.map_or(vec![*p2_puzzle_hash], |clawback| {
                vec![clawback.sender_puzzle_hash, clawback.receiver_puzzle_hash]
            }),
        }
    }

    pub fn receiver_custody_p2_puzzle_hash(&self) -> Option<Bytes32> {
        match self {
            Self::Launcher | Self::Unknown => None,
            Self::Clawback { info } => Some(info.receiver_puzzle_hash),
            Self::ClawbackV1 { info } => Some(info.receiver_puzzle_hash),
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
            Self::Option { info, clawback, .. } => clawback
                .map_or(Some(info.p2_puzzle_hash), |clawback| {
                    Some(clawback.receiver_puzzle_hash)
                }),
        }
    }

    pub fn subscribe(&self) -> bool {
        matches!(
            self,
            Self::Clawback { .. }
                | Self::ClawbackV1 { .. }
                | Self::Cat { .. }
                | Self::Did { .. }
                | Self::Nft { .. }
                | Self::Option { .. }
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

    // The database stores seconds as an i64, and Sage never creates clawbacks this far out.
    i64::try_from(seconds).ok()?;

    let clawback = ClawbackV2 {
        sender_puzzle_hash,
        receiver_puzzle_hash,
        seconds,
        amount,
        hinted,
    };

    Some(clawback)
}

#[cfg(test)]
mod tests {
    use chia_wallet_sdk::test::Simulator;

    use super::*;

    /// Sends a DID to the given p2 puzzle hash, with memos that hint the given puzzle hash
    /// and always claim a clawback, whether or not the DID is actually sent to one.
    fn transfer_did(p2_puzzle_hash: Bytes32, hint: Bytes32) -> anyhow::Result<ChildKind> {
        let mut sim = Simulator::new();
        let alice = sim.bls(1);
        let mut ctx = SpendContext::new();

        let (conditions, did) = Launcher::new(alice.coin.coin_id(), 1)
            .create_simple_did(&mut ctx, &StandardLayer::new(alice.pk))?;
        StandardLayer::new(alice.pk).spend(&mut ctx, alice.coin, conditions)?;
        ctx.take();

        let new_info = DidInfo {
            p2_puzzle_hash,
            ..did.info
        };

        let memos = ctx.memos(&(hint, ((Bytes32::new([7; 32]), (100_u64, ())), ())))?;
        did.spend_with(
            &mut ctx,
            &StandardLayer::new(alice.pk),
            Conditions::new().create_coin(new_info.inner_puzzle_hash().into(), 1, memos),
        )?;
        let child = did.child(p2_puzzle_hash, did.info.metadata, 1);

        let spend = ctx.take().remove(0);

        Ok(ChildKind::from_parent(
            spend.coin,
            &spend.puzzle_reveal,
            &spend.solution,
            child.coin,
        )?)
    }

    #[test]
    fn test_did_clawback_memos_must_match_puzzle() -> anyhow::Result<()> {
        let receiver = Bytes32::new([8; 32]);

        let ChildKind::Did { info, clawback, .. } = transfer_did(receiver, receiver)? else {
            panic!("expected a DID");
        };
        assert_eq!(info.p2_puzzle_hash, receiver);
        assert_eq!(clawback, None);

        let real = ClawbackV2::new(Bytes32::new([7; 32]), receiver, 100, 1, true);
        let ChildKind::Did { info, clawback, .. } =
            transfer_did(real.tree_hash().into(), receiver)?
        else {
            panic!("expected a DID");
        };
        assert_eq!(info.p2_puzzle_hash, real.tree_hash().into());
        assert_eq!(clawback, Some(real));

        Ok(())
    }

    #[test]
    fn test_did_hint_must_match_puzzle() -> anyhow::Result<()> {
        let kind = transfer_did(Bytes32::new([8; 32]), Bytes32::new([9; 32]))?;
        assert!(matches!(kind, ChildKind::Unknown));
        Ok(())
    }
}

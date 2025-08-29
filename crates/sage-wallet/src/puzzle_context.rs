use std::time::Duration;

use chia::{
    clvm_traits::{FromClvm, ToClvm},
    protocol::{Bytes32, Coin, CoinSpend, CoinState},
    puzzles::{
        nft::{NftOwnershipLayerSolution, NftStateLayerSolution},
        singleton::{LauncherSolution, SingletonSolution},
        Memos,
    },
};
use chia_puzzles::SINGLETON_LAUNCHER_HASH;
use chia_wallet_sdk::{
    driver::{DidInfo, NftInfo, OptionInfo, OptionMetadata, Puzzle},
    prelude::CreateCoin,
    types::{run_puzzle, Condition, Conditions},
};
use clvmr::{Allocator, NodePtr};
use tokio::time::sleep;
use tracing::warn;

use crate::{ChildKind, WalletError, WalletPeer};

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Default, Clone)]
pub enum PuzzleContext {
    #[default]
    None,
    Nft {
        minter_hash: Option<Bytes32>,
    },
    Option(OptionContext),
}

impl PuzzleContext {
    pub async fn fetch(
        peer: &WalletPeer,
        genesis_challenge: Bytes32,
        kind: &ChildKind,
    ) -> Result<Self, WalletError> {
        match kind {
            ChildKind::Nft { info, .. } => {
                let minter_hash =
                    fetch_minter_hash(peer, genesis_challenge, info.launcher_id).await?;

                Ok(Self::Nft { minter_hash })
            }
            ChildKind::Option { info, .. } => {
                let Some(context) = fetch_option(peer, genesis_challenge, info).await? else {
                    return Ok(Self::None);
                };

                Ok(Self::Option(context))
            }
            _ => Ok(Self::None),
        }
    }
}

pub async fn fetch_minter_hash(
    peer: &WalletPeer,
    genesis_challenge: Bytes32,
    launcher_id: Bytes32,
) -> Result<Option<Bytes32>, WalletError> {
    let mut did_id = None::<Bytes32>;
    let mut parent_id = launcher_id;

    for _ in 0..5 {
        let Some(parent_spend) = peer
            .fetch_optional_coin_spend(parent_id, genesis_challenge)
            .await?
        else {
            break;
        };

        let mut allocator = Allocator::new();

        let puzzle_reveal = parent_spend.puzzle_reveal.to_clvm(&mut allocator)?;
        let puzzle = Puzzle::parse(&allocator, puzzle_reveal);

        if let Some((did, _)) = DidInfo::parse(&allocator, puzzle).ok().flatten() {
            did_id = Some(did.launcher_id);
            break;
        }

        parent_id = parent_spend.coin.parent_coin_info;

        sleep(Duration::from_secs(1)).await;
    }

    if did_id.is_none() {
        let coin_spend = {
            let child = peer.fetch_singleton_child(launcher_id).await?;
            let spent_height = child.spent_height.ok_or(WalletError::PeerMisbehaved)?;
            let (puzzle_reveal, solution) = peer
                .fetch_puzzle_solution(child.coin.coin_id(), spent_height)
                .await?;
            CoinSpend::new(child.coin, puzzle_reveal, solution)
        };

        let mut allocator = Allocator::new();

        let puzzle_reveal = coin_spend.puzzle_reveal.to_clvm(&mut allocator)?;
        let solution = coin_spend.solution.to_clvm(&mut allocator)?;
        let puzzle = Puzzle::parse(&allocator, puzzle_reveal);

        if let Some((_nft_info, p2_puzzle)) = NftInfo::parse(&allocator, puzzle).ok().flatten() {
            if let Ok(solution) = SingletonSolution::<
                NftStateLayerSolution<NftOwnershipLayerSolution<NodePtr>>,
            >::from_clvm(&allocator, solution)
            {
                let p2_solution = solution.inner_solution.inner_solution.inner_solution;

                if let Ok(output) = run_puzzle(&mut allocator, p2_puzzle.ptr(), p2_solution) {
                    if let Ok(conditions) = Conditions::<NodePtr>::from_clvm(&allocator, output) {
                        did_id = conditions.into_iter().find_map(|cond| match cond {
                            Condition::TransferNft(transfer) => transfer.launcher_id,
                            _ => None,
                        });
                    }
                }
            }
        }
    }

    Ok(did_id)
}

#[derive(Debug, Clone)]
pub struct OptionContext {
    pub underlying_parent: Coin,
    pub underlying: CoinState,
    pub underlying_kind: ChildKind,
    pub metadata: OptionMetadata,
    pub creator_puzzle_hash: Bytes32,
}

pub async fn fetch_option(
    peer: &WalletPeer,
    genesis_challenge: Bytes32,
    info: &OptionInfo,
) -> Result<Option<OptionContext>, WalletError> {
    let Some(launcher) = peer
        .fetch_optional_coin_spend(info.launcher_id, genesis_challenge)
        .await?
    else {
        return Ok(None);
    };

    let mut allocator = Allocator::new();

    let solution = launcher.solution.to_clvm(&mut allocator)?;

    if launcher.coin.puzzle_hash != SINGLETON_LAUNCHER_HASH.into() {
        return Ok(None);
    }

    let Ok(launcher_solution) = LauncherSolution::<OptionMetadata>::from_clvm(&allocator, solution)
    else {
        return Ok(None);
    };

    let metadata = launcher_solution.key_value_list;

    let Some(launcher_parent) = peer
        .fetch_optional_coin_spend(launcher.coin.parent_coin_info, genesis_challenge)
        .await?
    else {
        return Ok(None);
    };

    let launcher_parent_puzzle = launcher_parent.puzzle_reveal.to_clvm(&mut allocator)?;
    let launcher_parent_solution = launcher_parent.solution.to_clvm(&mut allocator)?;
    let output = run_puzzle(
        &mut allocator,
        launcher_parent_puzzle,
        launcher_parent_solution,
    )?;
    let conditions = Vec::<Condition>::from_clvm(&allocator, output)?;

    let Some(CreateCoin {
        memos: Memos::Some(memos),
        ..
    }) = conditions
        .into_iter()
        .filter_map(Condition::into_create_coin)
        .find(|cc| {
            cc.puzzle_hash == SINGLETON_LAUNCHER_HASH.into() && cc.amount == launcher.coin.amount
        })
    else {
        return Ok(None);
    };

    let Ok((hint, _)) = <(Bytes32, NodePtr)>::from_clvm(&allocator, memos) else {
        return Ok(None);
    };

    let Some(underlying) = peer
        .fetch_optional_coin(info.underlying_coin_id, genesis_challenge)
        .await?
    else {
        return Ok(None);
    };

    let Some(parent) = peer
        .fetch_optional_coin_spend(underlying.coin.parent_coin_info, genesis_challenge)
        .await?
    else {
        return Ok(None);
    };

    let underlying_kind = match ChildKind::from_parent(
        parent.coin,
        &parent.puzzle_reveal,
        &parent.solution,
        underlying.coin,
    ) {
        Ok(kind) => kind,
        Err(error) => {
            warn!("Failed to parse underlying coin kind: {error}");
            return Ok(None);
        }
    };

    Ok(Some(OptionContext {
        underlying_parent: parent.coin,
        underlying,
        underlying_kind,
        metadata,
        creator_puzzle_hash: hint,
    }))
}

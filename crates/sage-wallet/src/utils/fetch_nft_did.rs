use std::collections::hash_map::RandomState;
use std::{collections::HashMap, time::Duration};

use chia::{
    clvm_traits::{FromClvm, ToClvm},
    protocol::{Bytes32, CoinSpend},
    puzzles::{
        nft::{NftOwnershipLayerSolution, NftStateLayerSolution},
        singleton::SingletonSolution,
    },
};
use chia_wallet_sdk::{
    driver::{DidInfo, HashedPtr, NftInfo, Puzzle},
    types::{run_puzzle, Condition, Conditions},
};
use clvmr::{Allocator, NodePtr};
use tokio::time::{sleep, timeout};

use crate::{WalletError, WalletPeer};

pub async fn fetch_nft_did(
    peer: &WalletPeer,
    genesis_challenge: Bytes32,
    launcher_id: Bytes32,
    pending_coin_spends: &HashMap<Bytes32, CoinSpend, RandomState>,
) -> Result<Option<Bytes32>, WalletError> {
    let mut did_id = None::<Bytes32>;
    let mut parent_id = launcher_id;

    for _ in 0..5 {
        let (fetched, parent_spend) =
            if let Some(coin_spend) = pending_coin_spends.get(&parent_id).cloned() {
                (false, coin_spend)
            } else {
                let Some(parent_spend) = timeout(
                    Duration::from_secs(15),
                    peer.fetch_optional_coin_spend(parent_id, genesis_challenge),
                )
                .await??
                else {
                    break;
                };
                (true, parent_spend)
            };

        let mut allocator = Allocator::new();

        let puzzle_reveal = parent_spend.puzzle_reveal.to_clvm(&mut allocator)?;
        let puzzle = Puzzle::parse(&allocator, puzzle_reveal);

        if let Some((did, _)) = DidInfo::<HashedPtr>::parse(&allocator, puzzle)
            .ok()
            .flatten()
        {
            did_id = Some(did.launcher_id);
            break;
        }

        parent_id = parent_spend.coin.parent_coin_info;

        if fetched {
            sleep(Duration::from_secs(1)).await;
        }
    }

    if did_id.is_none() {
        let coin_spend = if let Some(coin_spend) = pending_coin_spends
            .values()
            .find(|cs| cs.coin.parent_coin_info == launcher_id)
            .cloned()
        {
            coin_spend
        } else {
            let child = timeout(Duration::from_secs(5), peer.fetch_child(launcher_id)).await??;
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

        if let Some((_nft_info, p2_puzzle)) = NftInfo::<HashedPtr>::parse(&allocator, puzzle)
            .ok()
            .flatten()
        {
            if let Ok(solution) = SingletonSolution::<
                NftStateLayerSolution<NftOwnershipLayerSolution<NodePtr>>,
            >::from_clvm(&allocator, solution)
            {
                let p2_solution = solution.inner_solution.inner_solution.inner_solution;

                if let Ok(output) = run_puzzle(&mut allocator, p2_puzzle.ptr(), p2_solution) {
                    if let Ok(conditions) = Conditions::<NodePtr>::from_clvm(&allocator, output) {
                        did_id = conditions.into_iter().find_map(|cond| match cond {
                            Condition::TransferNft(transfer) => transfer.did_id,
                            _ => None,
                        });
                    }
                }
            }
        }
    }

    Ok(did_id)
}

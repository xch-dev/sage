use std::time::Duration;

use chia::{
    clvm_traits::{FromClvm, ToClvm},
    protocol::Bytes32,
    puzzles::{
        nft::{NftOwnershipLayerSolution, NftStateLayerSolution},
        singleton::SingletonSolution,
    },
};
use chia_wallet_sdk::{run_puzzle, Condition, Conditions, DidInfo, HashedPtr, NftInfo, Puzzle};
use clvmr::{Allocator, NodePtr};
use tokio::time::{sleep, timeout};

use crate::{WalletError, WalletPeer};

pub async fn fetch_nft_did(
    peer: &WalletPeer,
    genesis_challenge: Bytes32,
    launcher_id: Bytes32,
) -> Result<Option<Bytes32>, WalletError> {
    let mut did_id = None::<Bytes32>;
    let mut parent_id = launcher_id;

    for _ in 0..5 {
        let parent_spend = timeout(
            Duration::from_secs(15),
            peer.fetch_coin_spend(parent_id, genesis_challenge),
        )
        .await??;

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

        sleep(Duration::from_secs(1)).await;
    }

    if did_id.is_none() {
        let child = timeout(Duration::from_secs(5), peer.fetch_child(launcher_id)).await??;
        let spent_height = child.spent_height.ok_or(WalletError::PeerMisbehaved)?;
        let (puzzle_reveal, solution) = peer
            .fetch_puzzle_solution(child.coin.coin_id(), spent_height)
            .await?;

        let mut allocator = Allocator::new();

        let puzzle_reveal = puzzle_reveal.to_clvm(&mut allocator)?;
        let solution = solution.to_clvm(&mut allocator)?;
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

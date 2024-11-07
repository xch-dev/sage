use std::time::Duration;

use chia::{
    clvm_traits::{FromClvm, ToClvm},
    protocol::Bytes32,
    puzzles::{
        nft::{NftOwnershipLayerSolution, NftStateLayerSolution},
        singleton::SingletonSolution,
    },
};
use chia_wallet_sdk::{
    run_puzzle, Condition, Conditions, DidInfo, HashedPtr, NftInfo, Peer, Puzzle,
};
use clvmr::{Allocator, NodePtr};
use tokio::time::{sleep, timeout};

use crate::WalletError;

pub async fn fetch_nft_did(
    peer: &Peer,
    genesis_challenge: Bytes32,
    launcher_id: Bytes32,
) -> Result<Option<Bytes32>, WalletError> {
    let mut did_id = None::<Bytes32>;
    let mut parent_id = launcher_id;

    for _ in 0..5 {
        let Some(parent) = timeout(
            Duration::from_secs(5),
            peer.request_coin_state(vec![parent_id], None, genesis_challenge, false),
        )
        .await??
        .map_err(|_| WalletError::PeerMisbehaved)?
        .coin_states
        .into_iter()
        .next() else {
            break;
        };

        let height = parent.spent_height.ok_or(WalletError::PeerMisbehaved)?;

        let response = timeout(
            Duration::from_secs(5),
            peer.request_puzzle_and_solution(parent_id, height),
        )
        .await??
        .map_err(|_| WalletError::MissingSpend(parent_id))?;

        let mut allocator = Allocator::new();

        let puzzle_reveal = response.puzzle.to_clvm(&mut allocator)?;

        let puzzle = Puzzle::parse(&allocator, puzzle_reveal);

        if let Some((did, _)) = DidInfo::<HashedPtr>::parse(&allocator, puzzle)
            .ok()
            .flatten()
        {
            did_id = Some(did.launcher_id);
            break;
        }

        parent_id = parent.coin.parent_coin_info;

        sleep(Duration::from_secs(1)).await;
    }

    if did_id.is_none() {
        let Some(child) = timeout(Duration::from_secs(5), peer.request_children(launcher_id))
            .await??
            .coin_states
            .into_iter()
            .next()
        else {
            return Err(WalletError::MissingChild(launcher_id));
        };

        let child_id = child.coin.coin_id();
        let height = child.spent_height.ok_or(WalletError::PeerMisbehaved)?;

        let response = timeout(
            Duration::from_secs(5),
            peer.request_puzzle_and_solution(child_id, height),
        )
        .await??
        .map_err(|_| WalletError::MissingSpend(child_id))?;

        let mut allocator = Allocator::new();

        let puzzle_reveal = response.puzzle.to_clvm(&mut allocator)?;
        let solution = response.solution.to_clvm(&mut allocator)?;
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

use std::time::Duration;

use chia::{
    clvm_traits::{FromClvm, ToClvm},
    protocol::{Bytes32, Program},
};
use chia_wallet_sdk::driver::{HashedPtr, Nft, Puzzle};
use clvmr::Allocator;
use tokio::time::{sleep, timeout};

use crate::{RequestedNft, WalletError, WalletPeer};

pub async fn fetch_nft_offer_details(
    peer: &WalletPeer,
    launcher_id: Bytes32,
) -> Result<Option<RequestedNft>, WalletError> {
    let mut offer_details = None::<RequestedNft>;
    let mut current_id = launcher_id;
    let mut parent = None;

    loop {
        let Some(child) =
            timeout(Duration::from_secs(5), peer.try_fetch_child(current_id)).await??
        else {
            return Ok(None);
        };

        if child.spent_height.is_some() {
            parent = Some(child);
            current_id = child.coin.coin_id();
            sleep(Duration::from_secs(1)).await;
            continue;
        }

        let parent = parent.expect("parent not found");

        let (parent_puzzle_reveal, parent_solution) = timeout(
            Duration::from_secs(15),
            peer.fetch_puzzle_solution(
                parent.coin.coin_id(),
                parent.spent_height.ok_or(WalletError::PeerMisbehaved)?,
            ),
        )
        .await??;

        let mut allocator = Allocator::new();
        let parent_puzzle = parent_puzzle_reveal.to_clvm(&mut allocator)?;
        let parent_puzzle = Puzzle::parse(&allocator, parent_puzzle);
        let parent_solution = parent_solution.to_clvm(&mut allocator)?;

        if let Some(nft) = Nft::<HashedPtr>::parse_child(
            &mut allocator,
            parent.coin,
            parent_puzzle,
            parent_solution,
        )
        .ok()
        .flatten()
        {
            offer_details = Some(RequestedNft {
                metadata: Program::from_clvm(&allocator, nft.info.metadata.ptr())?,
                metadata_updater_puzzle_hash: nft.info.metadata_updater_puzzle_hash,
                royalty_puzzle_hash: nft.info.royalty_puzzle_hash,
                royalty_ten_thousandths: nft.info.royalty_ten_thousandths,
            });
            break;
        }

        break;
    }

    Ok(offer_details)
}

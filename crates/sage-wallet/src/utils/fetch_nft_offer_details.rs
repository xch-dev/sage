use std::time::Duration;

use chia::{
    clvm_traits::{FromClvm, ToClvm},
    protocol::{Bytes32, Program},
};
use chia_wallet_sdk::{HashedPtr, Nft, Puzzle};
use clvmr::Allocator;
use tokio::time::{sleep, timeout};

use crate::{NftOfferDetails, WalletError, WalletPeer};

pub async fn fetch_nft_offer_details(
    peer: &WalletPeer,
    launcher_id: Bytes32,
) -> Result<Option<NftOfferDetails>, WalletError> {
    let mut offer_details = None::<NftOfferDetails>;
    let mut current_id = launcher_id;

    loop {
        let Some(child) =
            timeout(Duration::from_secs(5), peer.try_fetch_child(current_id)).await??
        else {
            break;
        };

        let spent_height = child.spent_height.ok_or(WalletError::PeerMisbehaved)?;
        current_id = child.coin.coin_id();

        let (puzzle_reveal, solution) = timeout(
            Duration::from_secs(15),
            peer.fetch_puzzle_solution(current_id, spent_height),
        )
        .await??;

        let mut allocator = Allocator::new();

        let puzzle_reveal = puzzle_reveal.to_clvm(&mut allocator)?;
        let puzzle = Puzzle::parse(&allocator, puzzle_reveal);
        let solution = solution.to_clvm(&mut allocator)?;

        if let Some(nft) =
            Nft::<HashedPtr>::parse_child(&mut allocator, child.coin, puzzle, solution)
                .ok()
                .flatten()
        {
            offer_details = Some(NftOfferDetails {
                metadata: Program::from_clvm(&allocator, nft.info.metadata.ptr())?,
                metadata_updater_puzzle_hash: nft.info.metadata_updater_puzzle_hash,
                royalty_puzzle_hash: nft.info.royalty_puzzle_hash,
                royalty_ten_thousandths: nft.info.royalty_ten_thousandths,
            });
            break;
        }

        sleep(Duration::from_secs(1)).await;
    }

    Ok(offer_details)
}

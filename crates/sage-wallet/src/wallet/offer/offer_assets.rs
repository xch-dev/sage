use std::{collections::HashMap, time::Duration};

use chia::{
    clvm_traits::{FromClvm, ToClvm},
    protocol::{Bytes32, CoinState, Program},
    puzzles::nft::NftMetadata,
};
use chia_wallet_sdk::driver::{HashedPtr, Nft, Puzzle};
use clvmr::Allocator;
use sage_database::NftOfferInfo;
use tokio::time::{sleep, timeout};

use crate::{fetch_nft_did, insert_nft, Wallet, WalletError, WalletPeer};

impl Wallet {
    pub async fn fetch_offer_nft_info(
        &self,
        peer: Option<&WalletPeer>,
        launcher_id: Bytes32,
    ) -> Result<Option<NftOfferInfo>, WalletError> {
        let Some(peer) = peer else {
            if let Some(row) = self.db.offer_nft_info(launcher_id).await? {
                return Ok(Some(row));
            };

            return Ok(None);
        };

        let minter_did =
            fetch_nft_did(peer, self.genesis_challenge, launcher_id, &HashMap::new()).await?;

        let mut allocator = Allocator::new();
        let mut current_id = launcher_id;
        let mut parent = None;

        let nft = loop {
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
                break nft;
            }

            return Ok(None);
        };

        let metadata = Program::from_clvm(&allocator, nft.info.metadata.ptr())?;
        let parsed_metadata = NftMetadata::from_clvm(&allocator, nft.info.metadata.ptr()).ok();

        let mut tx = self.db.tx().await?;

        insert_nft(
            &mut tx,
            CoinState::new(nft.coin, None, None),
            None,
            nft.info.with_metadata(metadata.clone()),
            parsed_metadata,
            minter_did,
        )
        .await?;

        tx.commit().await?;

        let offer_details = Some(NftOfferInfo {
            metadata,
            metadata_updater_puzzle_hash: nft.info.metadata_updater_puzzle_hash,
            royalty_puzzle_hash: nft.info.royalty_puzzle_hash,
            royalty_basis_points: nft.info.royalty_basis_points,
        });

        Ok(offer_details)
    }
}

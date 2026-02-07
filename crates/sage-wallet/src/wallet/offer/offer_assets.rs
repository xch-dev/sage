use std::time::Duration;

use chia_wallet_sdk::{chia::puzzle_types::nft::NftMetadata, prelude::*};
use sage_database::{NftOfferInfo, OptionOfferInfo, SerializePrimitive};
use tokio::time::sleep;

use crate::{
    fetch_minter_hash, fetch_option, insert_nft, insert_option, PuzzleContext, Wallet, WalletError,
    WalletPeer,
};

impl Wallet {
    pub async fn fetch_offer_cat_hidden_puzzle_hash(
        &self,
        asset_id: Bytes32,
    ) -> Result<Option<Bytes32>, WalletError> {
        Ok(self
            .db
            .asset(asset_id)
            .await?
            .and_then(|asset| asset.hidden_puzzle_hash))
    }

    pub async fn fetch_offer_nft_info(
        &self,
        peer: Option<&WalletPeer>,
        launcher_id: Bytes32,
    ) -> Result<Option<NftOfferInfo>, WalletError> {
        let Some(peer) = peer else {
            if let Some(row) = self.db.offer_nft_info(launcher_id).await? {
                return Ok(Some(row));
            }

            return Ok(None);
        };

        let minter_hash = fetch_minter_hash(peer, self.genesis_challenge, launcher_id).await?;

        let mut allocator = Allocator::new();
        let mut current_id = launcher_id;
        let mut parent = None;

        let nft = loop {
            let Some(child) = peer.try_fetch_singleton_child(current_id).await? else {
                return Ok(None);
            };

            if child.spent_height.is_some() {
                parent = Some(child);
                current_id = child.coin.coin_id();
                sleep(Duration::from_secs(1)).await;
                continue;
            }

            let parent = parent.expect("parent not found");

            let (parent_puzzle_reveal, parent_solution) = peer
                .fetch_puzzle_solution(
                    parent.coin.coin_id(),
                    parent.spent_height.ok_or(WalletError::PeerMisbehaved)?,
                )
                .await?;

            let parent_puzzle = parent_puzzle_reveal.to_clvm(&mut allocator)?;
            let parent_puzzle = Puzzle::parse(&allocator, parent_puzzle);
            let parent_solution = parent_solution.to_clvm(&mut allocator)?;

            if let Some(nft) =
                Nft::parse_child(&mut allocator, parent.coin, parent_puzzle, parent_solution)
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
            nft.serialize(&allocator)?.info,
            parsed_metadata,
            PuzzleContext::Nft { minter_hash },
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

    pub async fn fetch_offer_option_info(
        &self,
        peer: Option<&WalletPeer>,
        launcher_id: Bytes32,
    ) -> Result<Option<OptionOfferInfo>, WalletError> {
        let Some(peer) = peer else {
            if let Some(row) = self.db.offer_option_info(launcher_id).await? {
                return Ok(Some(row));
            }

            return Ok(None);
        };

        let mut allocator = Allocator::new();
        let mut current_id = launcher_id;
        let mut parent = None;

        let option = loop {
            let Some(child) = peer.try_fetch_singleton_child(current_id).await? else {
                return Ok(None);
            };

            if child.spent_height.is_some() {
                parent = Some(child);
                current_id = child.coin.coin_id();
                sleep(Duration::from_secs(1)).await;
                continue;
            }

            let parent = parent.expect("parent not found");

            let (parent_puzzle_reveal, parent_solution) = peer
                .fetch_puzzle_solution(
                    parent.coin.coin_id(),
                    parent.spent_height.ok_or(WalletError::PeerMisbehaved)?,
                )
                .await?;

            let parent_puzzle = parent_puzzle_reveal.to_clvm(&mut allocator)?;
            let parent_puzzle = Puzzle::parse(&allocator, parent_puzzle);
            let parent_solution = parent_solution.to_clvm(&mut allocator)?;

            if let Some(option) = OptionContract::parse_child(
                &mut allocator,
                parent.coin,
                parent_puzzle,
                parent_solution,
            )
            .ok()
            .flatten()
            {
                break option;
            }

            return Ok(None);
        };

        let Some(context) = fetch_option(peer, self.genesis_challenge, &option.info).await? else {
            return Ok(None);
        };

        let mut tx = self.db.tx().await?;

        insert_option(
            &mut tx,
            CoinState::new(option.coin, None, None),
            None,
            option.info,
            context,
        )
        .await?;

        tx.commit().await?;

        let offer_details = Some(OptionOfferInfo {
            underlying_coin_hash: option.info.underlying_coin_id,
            underlying_delegated_puzzle_hash: option.info.underlying_delegated_puzzle_hash,
        });

        Ok(offer_details)
    }
}

use std::{collections::hash_map::Entry, time::Duration};

use chia::{clvm_traits::FromClvm, protocol::Bytes32, puzzles::nft::NftMetadata};
use chia_wallet_sdk::types::TESTNET11_CONSTANTS;
use clvmr::{Allocator, NodePtr};
use sage_assets::{fetch_uris_with_hash, DexieCat};
use sage_database::{Asset, AssetKind};
use tokio::time::timeout;

use crate::{extract_nft_data, ConfirmationInfo, Error, ExtractedNftData, Result, Sage};

impl Sage {
    pub async fn cache_cat(
        &self,
        asset_id: Bytes32,
        hidden_puzzle_hash: Option<Bytes32>,
    ) -> Result<Asset> {
        let wallet = self.wallet()?;

        if let Some(asset) = wallet.db.asset(asset_id).await? {
            return Ok(asset);
        }

        let testnet = self.network().genesis_challenge == TESTNET11_CONSTANTS.genesis_challenge;

        let asset = if let Ok(Ok(asset)) =
            timeout(Duration::from_secs(5), DexieCat::fetch(asset_id, testnet)).await
        {
            Asset {
                hash: asset_id,
                name: asset.name,
                ticker: asset.ticker,
                precision: 3,
                icon_url: asset.icon_url,
                description: asset.description,
                is_sensitive_content: false,
                is_visible: true,
                hidden_puzzle_hash: asset.hidden_puzzle_hash.or(hidden_puzzle_hash),
                kind: AssetKind::Token,
            }
        } else {
            Asset {
                hash: asset_id,
                name: None,
                ticker: None,
                precision: 3,
                icon_url: None,
                description: None,
                is_sensitive_content: false,
                is_visible: true,
                hidden_puzzle_hash,
                kind: AssetKind::Token,
            }
        };

        wallet.db.insert_asset(asset.clone()).await?;

        let mut tx = wallet.db.tx().await?;

        if tx.existing_hidden_puzzle_hash(asset_id).await?.is_none() {
            tx.update_hidden_puzzle_hash(asset_id, asset.hidden_puzzle_hash.or(hidden_puzzle_hash))
                .await?;
        }

        tx.commit().await?;

        Ok(asset)
    }

    pub async fn cache_nft(
        &self,
        allocator: &Allocator,
        launcher_id: Bytes32,
        nft_metadata: NodePtr,
        confirmation_info: &mut ConfirmationInfo,
    ) -> Result<Asset> {
        let wallet = self.wallet()?;

        if let Some(asset) = wallet.db.asset(launcher_id).await? {
            return Ok(asset);
        }

        let info = if let Ok(metadata) = NftMetadata::from_clvm(allocator, nft_metadata) {
            let testnet = self.network().genesis_challenge == TESTNET11_CONSTANTS.genesis_challenge;

            if let Some(hash) = metadata.data_hash {
                if let Entry::Vacant(entry) = confirmation_info.nft_data.entry(hash) {
                    if let Ok(Some(data)) = timeout(
                        Duration::from_secs(10),
                        fetch_uris_with_hash(metadata.data_uris.clone(), hash, testnet),
                    )
                    .await
                    {
                        entry.insert(data);
                    }
                }
            }

            if let Some(hash) = metadata.metadata_hash {
                if let Entry::Vacant(entry) = confirmation_info.nft_data.entry(hash) {
                    if let Ok(Some(data)) = timeout(
                        Duration::from_secs(10),
                        fetch_uris_with_hash(metadata.metadata_uris.clone(), hash, testnet),
                    )
                    .await
                    {
                        entry.insert(data);
                    }
                }
            }

            extract_nft_data(Some(&wallet.db), Some(metadata), confirmation_info).await?
        } else {
            ExtractedNftData::default()
        };

        let asset = Asset {
            hash: launcher_id,
            name: info.name,
            ticker: None,
            precision: 1,
            icon_url: info.icon_url,
            description: info.description,
            is_sensitive_content: info.is_sensitive_content,
            is_visible: true,
            hidden_puzzle_hash: None,
            kind: AssetKind::Nft,
        };

        wallet.db.insert_asset(asset.clone()).await?;

        Ok(asset)
    }

    pub async fn cache_option(&self, launcher_id: Bytes32) -> Result<Asset> {
        let wallet = self.wallet()?;

        let peer = self.peer_state.lock().await.acquire_peer();

        wallet
            .fetch_offer_option_info(peer.as_ref(), launcher_id)
            .await?;

        let Some(asset) = wallet.db.asset(launcher_id).await? else {
            return Err(Error::MissingOption(launcher_id));
        };

        Ok(asset)
    }
}

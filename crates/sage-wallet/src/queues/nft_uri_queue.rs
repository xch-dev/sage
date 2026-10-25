use std::{collections::HashMap, time::Duration};

use chia_wallet_sdk::prelude::*;
use futures_lite::StreamExt;
use futures_util::stream::FuturesUnordered;
use sage_assets::{Data, base64_data_uri, fetch_mintgarden_archive, fetch_uri_with_hash};
use sage_database::{Database, DatabaseTx, FileUri, NftMetadataInfo, ResizedImageKind};
use tokio::{sync::mpsc, time::sleep};
use tracing::{debug, info, warn};

use crate::{SyncEvent, WalletError, compute_nft_info};
use chia_wallet_sdk::types::TESTNET11_CONSTANTS;
use sage_config::Network;

#[derive(Debug)]
pub struct NftUriQueue {
    db: Database,
    sync_sender: mpsc::Sender<SyncEvent>,
    network: Network,
}

impl NftUriQueue {
    pub fn new(db: Database, sync_sender: mpsc::Sender<SyncEvent>, network: Network) -> Self {
        Self {
            db,
            sync_sender,
            network,
        }
    }

    pub async fn start(self, delay: Duration) -> Result<(), WalletError> {
        loop {
            self.process_batch().await?;
            sleep(delay).await;
        }
    }

    async fn process_batch(&self) -> Result<(), WalletError> {
        let batch = self.db.candidates_for_download(60 * 60 * 24, 3, 25).await?;

        if batch.is_empty() {
            return Ok(());
        }

        info!("Processing batch of {} NFT URIs", batch.len());

        let mut groups: HashMap<Bytes32, Vec<FileUri>> = HashMap::new();

        for item in batch {
            groups.entry(item.hash).or_default().push(item);
        }

        let mut futures = FuturesUnordered::new();
        let testnet = self.network.genesis_challenge == TESTNET11_CONSTANTS.genesis_challenge;

        for (hash, items) in groups {
            futures.push(fetch_file(hash, items, testnet));
        }

        while let Some((hash, outcome)) = futures.next().await {
            let mut tx = self.db.tx().await?;

            match outcome {
                Outcome::Archived { data, items } => {
                    save_data(&mut tx, hash, data, true).await?;

                    for item in items {
                        tx.update_checked_uri(hash, item.uri).await?;
                    }
                }
                Outcome::Fetched(results) => {
                    // Prefer a file that matches the hash, but fall back to one
                    // that doesn't so that there's at least something to show.
                    let best = results
                        .iter()
                        .filter_map(|(_, result)| result.as_ref().ok())
                        .min_by_key(|data| data.hash != hash)
                        .cloned();

                    if let Some(data) = best {
                        let is_hash_match = data.hash == hash;
                        save_data(&mut tx, hash, data, is_hash_match).await?;
                    }

                    for (item, result) in results {
                        match result {
                            Ok(data) => {
                                if data.hash != hash {
                                    warn!(
                                        "Hash mismatch for URI {} (expected {} but found {})",
                                        item.uri, hash, data.hash
                                    );
                                }

                                tx.update_checked_uri(hash, item.uri).await?;
                            }
                            Err(error) => {
                                debug!("Error fetching URI {}: {error}", item.uri);
                                tx.update_failed_uri(hash, item.uri).await?;
                            }
                        }
                    }
                }
            }

            tx.commit().await?;
        }

        self.sync_sender.send(SyncEvent::NftData).await.ok();

        Ok(())
    }
}

#[derive(Debug)]
enum Outcome {
    Archived { data: Data, items: Vec<FileUri> },
    Fetched(Vec<(FileUri, Result<Data, String>)>),
}

/// Tries MintGarden's archive first, since it's content addressed and will
/// work even if the NFT's own URIs have gone offline. If it doesn't have the
/// file, falls back to fetching each of the NFT's URIs.
async fn fetch_file(hash: Bytes32, items: Vec<FileUri>, testnet: bool) -> (Bytes32, Outcome) {
    match fetch_mintgarden_archive(hash, testnet).await {
        Ok(Some(data)) => return (hash, Outcome::Archived { data, items }),
        Ok(None) => {}
        Err(error) => debug!("Error fetching {hash} from MintGarden archive: {error}"),
    }

    let mut futures = FuturesUnordered::new();

    for item in items {
        futures.push(async move {
            let result = fetch_uri_with_hash(item.uri.clone(), hash, testnet)
                .await
                .map_err(|error| error.to_string());

            (item, result)
        });
    }

    let mut results = Vec::new();

    while let Some(result) = futures.next().await {
        results.push(result);
    }

    (hash, Outcome::Fetched(results))
}

async fn save_data(
    tx: &mut DatabaseTx<'_>,
    hash: Bytes32,
    data: Data,
    is_hash_match: bool,
) -> Result<(), WalletError> {
    let icon_url = data
        .thumbnail
        .as_ref()
        .map(|thumbnail| base64_data_uri(&thumbnail.icon, "image/png"));

    if let Some(icon_url) = icon_url {
        tx.update_nft_data_hash_urls(hash, icon_url).await?;
    }

    for nft in tx.nfts_with_metadata_hash(hash).await? {
        let info = compute_nft_info(nft.minter_hash, &data.blob);

        let collection_id = info.collection.as_ref().map(|col| col.hash);

        if let Some(collection) = info.collection {
            tx.insert_collection(collection).await?;
        }

        tx.update_nft_metadata(
            nft.hash,
            NftMetadataInfo {
                name: info.name,
                description: info.description,
                is_sensitive_content: info.sensitive_content,
                collection_id: collection_id.unwrap_or_default(),
            },
        )
        .await?;
    }

    tx.update_file(hash, data.blob, data.mime_type, is_hash_match)
        .await?;

    if let Some(thumbnail) = data.thumbnail {
        tx.insert_resized_image(hash, ResizedImageKind::Icon, thumbnail.icon)
            .await?;

        tx.insert_resized_image(hash, ResizedImageKind::Thumbnail, thumbnail.thumbnail)
            .await?;
    }

    Ok(())
}

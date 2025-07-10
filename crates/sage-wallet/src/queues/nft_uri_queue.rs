use std::time::Duration;

use futures_lite::StreamExt;
use futures_util::stream::FuturesUnordered;
use sage_assets::{base64_data_uri, fetch_uri};
use sage_database::{Database, NftMetadataInfo, ResizedImageKind};
use tokio::{
    sync::mpsc,
    time::{sleep, timeout},
};
use tracing::{debug, info, warn};

use crate::{compute_nft_info, SyncEvent, WalletError};

#[derive(Debug)]
pub struct NftUriQueue {
    db: Database,
    sync_sender: mpsc::Sender<SyncEvent>,
}

impl NftUriQueue {
    pub fn new(db: Database, sync_sender: mpsc::Sender<SyncEvent>) -> Self {
        Self { db, sync_sender }
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

        let mut futures = FuturesUnordered::new();

        for item in batch {
            futures.push(async move {
                let result = timeout(Duration::from_secs(15), fetch_uri(item.uri.clone())).await;
                (item, result)
            });
        }

        while let Some((item, result)) = futures.next().await {
            let mut tx = self.db.tx().await?;

            match result {
                Ok(Ok(data)) => {
                    let is_hash_match = data.hash == item.hash;

                    if !is_hash_match {
                        warn!(
                            "Hash mismatch for URI {} (expected {} but found {})",
                            item.uri, item.hash, data.hash
                        );
                    }

                    let icon_url = data
                        .thumbnail
                        .as_ref()
                        .map(|thumbnail| base64_data_uri(&thumbnail.icon, &data.mime_type));

                    for nft in tx.nfts_with_metadata_hash(item.hash).await? {
                        let info = compute_nft_info(nft.minter_hash, &data.blob);

                        let collection_id = info.collection.as_ref().map(|col| col.hash);

                        if let Some(collection) = info.collection {
                            tx.insert_collection(collection).await?;
                        }

                        tx.update_nft_metadata(
                            nft.hash,
                            NftMetadataInfo {
                                name: info.name,
                                icon_url: icon_url.clone(),
                                description: info.description,
                                is_sensitive_content: info.sensitive_content,
                                collection_id: collection_id.unwrap_or_default(),
                            },
                        )
                        .await?;
                    }

                    tx.update_file(item.hash, data.blob, data.mime_type, is_hash_match)
                        .await?;

                    if let Some(thumbnail) = data.thumbnail {
                        tx.insert_resized_image(item.hash, ResizedImageKind::Icon, thumbnail.icon)
                            .await?;

                        tx.insert_resized_image(
                            item.hash,
                            ResizedImageKind::Thumbnail,
                            thumbnail.thumbnail,
                        )
                        .await?;
                    }

                    tx.update_checked_uri(item.hash, item.uri).await?;
                }
                Ok(Err(error)) => {
                    debug!("Error fetching URI {}: {error}", item.uri);
                    tx.update_failed_uri(item.hash, item.uri).await?;
                }
                Err(_error) => {
                    debug!("Timed out fetching URI {}", item.uri);
                    tx.update_failed_uri(item.hash, item.uri).await?;
                }
            }

            tx.commit().await?;
        }

        self.sync_sender.send(SyncEvent::NftData).await.ok();

        Ok(())
    }
}

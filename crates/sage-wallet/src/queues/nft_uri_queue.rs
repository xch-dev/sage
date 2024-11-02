use std::time::Duration;

use futures_lite::StreamExt;
use futures_util::stream::FuturesUnordered;
use sage_database::{Database, NftData};
use tokio::{sync::mpsc, time::sleep};
use tracing::{debug, info};

use crate::{compute_nft_info, fetch_uri, SyncEvent, WalletError};

#[derive(Debug)]
pub struct NftUriQueue {
    db: Database,
    sync_sender: mpsc::Sender<SyncEvent>,
}

impl NftUriQueue {
    pub fn new(db: Database, sync_sender: mpsc::Sender<SyncEvent>) -> Self {
        Self { db, sync_sender }
    }

    pub async fn start(self) -> Result<(), WalletError> {
        loop {
            self.process_batch().await?;
            sleep(Duration::from_secs(1)).await;
        }
    }

    async fn process_batch(&self) -> Result<(), WalletError> {
        let batch = self.db.unchecked_nft_uris(30).await?;

        if batch.is_empty() {
            return Ok(());
        }

        info!("Processing batch of {} NFT URIs", batch.len());

        let mut futures = FuturesUnordered::new();

        for item in batch {
            futures.push(async move {
                let result =
                    fetch_uri(&item.uri, Duration::from_secs(15), Duration::from_secs(15)).await;
                (item, result)
            });
        }

        while let Some((item, result)) = futures.next().await {
            let mut tx = self.db.tx().await?;

            match result {
                Ok(data) => {
                    if data.hash == item.hash {
                        if tx.fetch_nft_data(item.hash).await?.is_none() {
                            tx.insert_nft_data(
                                item.hash,
                                NftData {
                                    mime_type: data.mime_type,
                                    blob: data.blob.clone(),
                                },
                            )
                            .await?;

                            let nfts = tx.nfts_by_metadata_hash(item.hash).await?;

                            for mut nft in nfts {
                                let info = compute_nft_info(nft.minter_did, Some(&data.blob));

                                nft.sensitive_content = info.sensitive_content;
                                nft.name = info.name;
                                nft.collection_id =
                                    info.collection.as_ref().map(|col| col.collection_id);

                                if let Some(collection) = info.collection {
                                    tx.insert_nft_collection(collection).await?;
                                }

                                tx.insert_nft(nft).await?;
                            }
                        }
                    } else {
                        debug!(
                            "Hash mismatch for URI {} (expected {} but found {})",
                            item.uri, item.hash, data.hash
                        );
                    }
                }
                Err(error) => {
                    debug!("{error}");
                }
            };

            tx.set_nft_uri_checked(item.uri, item.hash).await?;

            tx.commit().await?;
        }

        self.sync_sender.send(SyncEvent::NftData).await.ok();

        Ok(())
    }
}

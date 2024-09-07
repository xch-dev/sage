use std::time::Duration;

use futures_lite::StreamExt;
use futures_util::stream::FuturesUnordered;
use reqwest::header::CONTENT_TYPE;
use sage_database::Database;
use tokio::{
    sync::mpsc,
    time::{sleep, timeout},
};
use tracing::{debug, info};

use crate::{SyncEvent, WalletError};

#[derive(Debug)]
pub struct NftQueue {
    db: Database,
    sync_sender: mpsc::Sender<SyncEvent>,
}

impl NftQueue {
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
                let response = timeout(Duration::from_secs(3), reqwest::get(&item.uri)).await;
                let result = match response {
                    Ok(Ok(response)) => {
                        let mime_type = response.headers().get(CONTENT_TYPE).cloned();
                        let data = timeout(Duration::from_secs(3), response.bytes()).await;
                        match data {
                            Ok(Ok(data)) => Some((data.to_vec(), mime_type)),
                            Ok(Err(error)) => {
                                debug!(
                                    "Failed to consume response bytes for NFT URI {}: {}",
                                    item.uri, error
                                );
                                None
                            }
                            Err(_) => {
                                debug!(
                                    "Timed out consuming response bytes for NFT URI {}",
                                    item.uri
                                );
                                None
                            }
                        }
                    }
                    Ok(Err(error)) => {
                        debug!("Failed to fetch NFT data for {}: {}", item.uri, error);
                        None
                    }
                    Err(_) => {
                        debug!("Timed out fetching NFT data for {}", item.uri);
                        None
                    }
                };
                (item, result)
            });
        }

        while let Some((item, result)) = futures.next().await {
            let mut tx = self.db.tx().await?;

            if let Some((data, mime_type)) = result {
                if let Some(mime_type) = mime_type {
                    if let Ok(mime_type) = mime_type.to_str() {
                        tx.insert_nft_data(item.hash, data, mime_type.to_string())
                            .await?;
                    } else {
                        debug!("Invalid content type for NFT URI {}", item.uri);
                    }
                } else {
                    debug!("No content type for NFT URI {}", item.uri);
                }
            }

            tx.mark_nft_uri_checked(item.uri, item.hash).await?;

            tx.commit().await?;
        }

        self.sync_sender.send(SyncEvent::NftUpdate).await.ok();

        Ok(())
    }
}

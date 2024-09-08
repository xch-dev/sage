use std::time::Duration;

use futures_lite::StreamExt;
use futures_util::stream::FuturesUnordered;
use reqwest::header::CONTENT_TYPE;
use sage_database::{Database, NftData};
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
                let result =
                    fetch_uri(&item.uri, Duration::from_secs(3), Duration::from_secs(3)).await;
                (item, result)
            });
        }

        while let Some((item, result)) = futures.next().await {
            let mut tx = self.db.tx().await?;

            if let Some(nft_data) = result {
                tx.insert_nft_data(item.hash, nft_data).await?;
            }

            tx.mark_nft_uri_checked(item.uri, item.hash).await?;

            tx.commit().await?;
        }

        self.sync_sender.send(SyncEvent::NftUpdate).await.ok();

        Ok(())
    }
}

async fn fetch_uri(
    uri: &str,
    request_timeout: Duration,
    stream_timeout: Duration,
) -> Option<NftData> {
    let response = match timeout(request_timeout, reqwest::get(uri)).await {
        Ok(Ok(response)) => response,
        Ok(Err(error)) => {
            debug!("Failed to fetch NFT data for {}: {}", uri, error);
            return None;
        }
        Err(_) => {
            debug!("Timed out fetching NFT data for {}", uri);
            return None;
        }
    };

    let Some(mime_type) = response
        .headers()
        .get(CONTENT_TYPE)
        .cloned()
        .and_then(|value| value.to_str().map(ToString::to_string).ok())
    else {
        debug!("Invalid or missing content type for NFT URI {}", uri);
        return None;
    };

    let blob = match timeout(stream_timeout, response.bytes()).await {
        Ok(Ok(data)) => data.to_vec(),
        Ok(Err(error)) => {
            debug!(
                "Failed to consume response bytes for NFT URI {}: {}",
                uri, error
            );
            return None;
        }
        Err(_) => {
            debug!("Timed out consuming response bytes for NFT URI {}", uri);
            return None;
        }
    };

    Some(NftData { blob, mime_type })
}

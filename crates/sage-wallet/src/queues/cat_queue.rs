use std::time::Duration;

use chia::protocol::Bytes32;
use sage_database::{CatRow, Database};
use serde::Deserialize;
use tokio::{
    sync::mpsc,
    time::{sleep, timeout},
};
use tracing::info;

use crate::{SyncError, SyncEvent, WalletError};

#[derive(Deserialize)]
struct Response {
    data: ResponseData,
}

#[derive(Deserialize)]
struct ResponseData {
    name: Option<String>,
    symbol: Option<String>,
    description: Option<String>,
    preview_url: Option<String>,
}

#[derive(Debug)]
pub struct CatQueue {
    db: Database,
    sync_sender: mpsc::Sender<SyncEvent>,
}

impl CatQueue {
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
        let Some(asset_id) = self.db.unidentified_cat().await? else {
            return Ok(());
        };

        info!(
            "Looking up CAT with asset id {} from spacescan.io",
            asset_id
        );

        let response = match timeout(Duration::from_secs(10), lookup_cat(asset_id)).await {
            Ok(Ok(response)) => response,
            Ok(Err(error)) => {
                info!("Failed to fetch CAT: {:?}", error);
                Response {
                    data: ResponseData {
                        name: None,
                        symbol: None,
                        description: None,
                        preview_url: None,
                    },
                }
            }
            Err(_) => {
                info!("Timeout fetching CAT");
                Response {
                    data: ResponseData {
                        name: None,
                        symbol: None,
                        description: None,
                        preview_url: None,
                    },
                }
            }
        };

        self.db
            .update_cat(CatRow {
                asset_id,
                name: response.data.name,
                ticker: response.data.symbol,
                description: response.data.description,
                icon_url: response.data.preview_url,
                precision: 3,
            })
            .await?;

        self.sync_sender.send(SyncEvent::CatUpdate).await.ok();

        Ok(())
    }
}

async fn lookup_cat(asset_id: Bytes32) -> Result<Response, SyncError> {
    let response = timeout(
        Duration::from_secs(10),
        reqwest::get(format!("https://api-fin.spacescan.io/cat/info/{asset_id}")),
    )
    .await
    .map_err(|_| SyncError::Timeout)?
    .map_err(|error| SyncError::FetchCat(asset_id, error))?;

    let response = response
        .json::<Response>()
        .await
        .map_err(|error| SyncError::FetchCat(asset_id, error))?;

    Ok(response)
}

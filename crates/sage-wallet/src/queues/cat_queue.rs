use std::time::Duration;

use chia::protocol::Bytes32;
use sage_database::{CatRow, Database};
use serde::Deserialize;
use tokio::{
    sync::mpsc,
    time::{sleep, timeout},
};
use tracing::info;

use crate::{SyncEvent, WalletError};

#[derive(Deserialize)]
struct Response {
    assets: Vec<AssetData>,
}

#[derive(Deserialize, Clone)]
struct AssetData {
    name: Option<String>,
    code: Option<String>,
    description: Option<String>,
}

#[derive(Debug)]
pub struct CatQueue {
    db: Database,
    testnet: bool,
    sync_sender: mpsc::Sender<SyncEvent>,
}

impl CatQueue {
    pub fn new(db: Database, testnet: bool, sync_sender: mpsc::Sender<SyncEvent>) -> Self {
        Self {
            db,
            testnet,
            sync_sender,
        }
    }

    pub async fn start(self, delay: Duration) -> Result<(), WalletError> {
        loop {
            self.process_batch().await?;
            sleep(delay).await;
        }
    }

    async fn process_batch(&self) -> Result<(), WalletError> {
        let Some(asset_id) = self.db.unfetched_cat().await? else {
            return Ok(());
        };

        info!(
            "Looking up CAT with asset id {} from spacescan.io",
            asset_id
        );

        let asset = match timeout(Duration::from_secs(10), lookup_cat(asset_id, self.testnet)).await
        {
            Ok(Ok(response)) => response.assets.first().cloned().unwrap_or(AssetData {
                name: None,
                code: None,
                description: None,
            }),
            Ok(Err(error)) => {
                info!("Failed to fetch CAT: {:?}", error);
                AssetData {
                    name: None,
                    code: None,
                    description: None,
                }
            }
            Err(_) => {
                info!("Timeout fetching CAT");
                AssetData {
                    name: None,
                    code: None,
                    description: None,
                }
            }
        };

        let dexie_image_base_url = if self.testnet {
            "https://icons-testnet.dexie.space"
        } else {
            "https://icons.dexie.space"
        };

        self.db
            .update_cat(CatRow {
                asset_id,
                name: asset.name,
                ticker: asset.code,
                description: asset.description,
                icon: Some(format!("{dexie_image_base_url}/{asset_id}.webp")),
                visible: true,
                fetched: true,
            })
            .await?;

        self.sync_sender.send(SyncEvent::CatInfo).await.ok();

        Ok(())
    }
}

async fn lookup_cat(asset_id: Bytes32, testnet: bool) -> Result<Response, WalletError> {
    let dexie_base_url = if testnet {
        "https://api-testnet.dexie.space/v1"
    } else {
        "https://api.dexie.space/v1"
    };

    let response = timeout(
        Duration::from_secs(10),
        reqwest::get(format!(
            "{dexie_base_url}/assets?page_size=25&page=1&type=all&code={asset_id}"
        )),
    )
    .await??
    .json::<Response>()
    .await?;

    Ok(response)
}

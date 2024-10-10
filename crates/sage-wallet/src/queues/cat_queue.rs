use std::time::Duration;

use chia::protocol::Bytes32;
use chia_wallet_sdk::{Network, TESTNET11_CONSTANTS};
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
    network: Network,
    sync_sender: mpsc::Sender<SyncEvent>,
}

impl CatQueue {
    pub fn new(db: Database, network: Network, sync_sender: mpsc::Sender<SyncEvent>) -> Self {
        Self {
            db,
            network,
            sync_sender,
        }
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

        let asset =
            match timeout(Duration::from_secs(10), lookup_cat(&self.network, asset_id)).await {
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

        let dexie_image_base_url =
            if self.network.genesis_challenge == TESTNET11_CONSTANTS.genesis_challenge {
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
                icon_url: Some(format!("{dexie_image_base_url}/{asset_id}.webp")),
                visible: true,
            })
            .await?;

        self.sync_sender.send(SyncEvent::CatUpdate).await.ok();

        Ok(())
    }
}

async fn lookup_cat(network: &Network, asset_id: Bytes32) -> Result<Response, SyncError> {
    let dexie_base_url = if network.genesis_challenge == TESTNET11_CONSTANTS.genesis_challenge {
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
    .await
    .map_err(|_| SyncError::Timeout)?
    .map_err(|error| SyncError::FetchCat(asset_id, error))?;

    let response = response
        .json::<Response>()
        .await
        .map_err(|error| SyncError::FetchCat(asset_id, error))?;

    Ok(response)
}

use std::{path::PathBuf, sync::Arc, time::Duration};

use chia::protocol::Bytes32;
use chia_wallet_sdk::TESTNET11_CONSTANTS;
use sage_api::SyncEvent;
use sage_config::Assets;
use sage_database::Database;
use serde::Deserialize;
use tauri::{AppHandle, Emitter};
use tokio::{
    sync::Mutex,
    time::{sleep, timeout},
};
use tracing::info;

use crate::{Error, Result};

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
    app_handle: AppHandle,
    db: Database,
    genesis_challenge: Bytes32,
    assets: Arc<Mutex<Assets>>,
    assets_path: PathBuf,
}

impl CatQueue {
    pub fn new(
        db: Database,
        app_handle: AppHandle,
        genesis_challenge: Bytes32,
        assets: Arc<Mutex<Assets>>,
        assets_path: PathBuf,
    ) -> Self {
        Self {
            app_handle,
            db,
            genesis_challenge,
            assets,
            assets_path,
        }
    }

    pub async fn start(self) -> Result<()> {
        loop {
            self.process_batch().await?;
            sleep(Duration::from_secs(10)).await;
        }
    }

    async fn process_batch(&self) -> Result<()> {
        let asset_ids = self.db.asset_ids().await?;
        let assets = self.assets.lock().await.clone();

        for asset_id in asset_ids
            .into_iter()
            .filter(|id| !assets.tokens.contains_key(&hex::encode(id)))
        {
            info!(
                "Looking up CAT with asset id {} from spacescan.io",
                asset_id
            );

            let asset = match timeout(
                Duration::from_secs(10),
                lookup_cat(self.genesis_challenge, asset_id),
            )
            .await
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

            let dexie_image_base_url =
                if self.genesis_challenge == TESTNET11_CONSTANTS.genesis_challenge {
                    "https://icons-testnet.dexie.space"
                } else {
                    "https://icons.dexie.space"
                };

            {
                let mut assets = self.assets.lock().await;

                let saved = assets.tokens.entry(hex::encode(asset_id)).or_default();

                saved.name = asset.name;
                saved.ticker = asset.code;
                saved.description = asset.description;
                saved.icon_url = Some(format!("{dexie_image_base_url}/{asset_id}.webp"));
                saved.hidden = false;

                assets.save(&self.assets_path)?;
            }

            self.app_handle.emit("sync-event", SyncEvent::CatInfo).ok();

            sleep(Duration::from_secs(1)).await;
        }

        Ok(())
    }
}

async fn lookup_cat(genesis_challenge: Bytes32, asset_id: Bytes32) -> Result<Response> {
    let dexie_base_url = if genesis_challenge == TESTNET11_CONSTANTS.genesis_challenge {
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
    .map_err(|_| Error::fetch_cat(asset_id))?
    .map_err(|_| Error::fetch_cat(asset_id))?;

    let response = response
        .json::<Response>()
        .await
        .map_err(|_| Error::fetch_cat(asset_id))?;

    Ok(response)
}

use std::time::Duration;

use chia::protocol::Bytes32;
use serde::Deserialize;
use tokio::time::timeout;
use tracing::info;

use crate::WalletError;

#[derive(Debug, Default, Clone)]
pub struct FetchedCatDetails {
    pub name: Option<String>,
    pub ticker: Option<String>,
    pub description: Option<String>,
    pub icon_url: Option<String>,
}

pub async fn lookup_cat(
    asset_id: Bytes32,
    testnet: bool,
) -> Result<FetchedCatDetails, WalletError> {
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

    let dexie_base_url = if testnet {
        "https://api-testnet.dexie.space/v1"
    } else {
        "https://api.dexie.space/v1"
    };

    let dexie_image_base_url = if testnet {
        "https://icons-testnet.dexie.space"
    } else {
        "https://icons.dexie.space"
    };

    let mut response = reqwest::get(format!(
        "{dexie_base_url}/assets?page_size=25&page=1&type=all&code={asset_id}"
    ))
    .await?
    .json::<Response>()
    .await?;

    if response.assets.is_empty() {
        return Ok(FetchedCatDetails::default());
    }

    let asset = response.assets.remove(0);

    Ok(FetchedCatDetails {
        name: asset.name,
        ticker: asset.code,
        description: asset.description,
        icon_url: Some(format!("{dexie_image_base_url}/{asset_id}.webp")),
    })
}

pub async fn try_lookup_cat(asset_id: Bytes32, testnet: bool) -> FetchedCatDetails {
    match timeout(Duration::from_secs(10), lookup_cat(asset_id, testnet)).await {
        Ok(Ok(asset)) => asset,
        Ok(Err(error)) => {
            info!("Failed to fetch CAT: {:?}", error);
            FetchedCatDetails::default()
        }
        Err(_) => {
            info!("Timeout fetching CAT");
            FetchedCatDetails::default()
        }
    }
}

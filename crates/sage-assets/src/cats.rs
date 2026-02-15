use chia_wallet_sdk::prelude::*;
use serde::Deserialize;

use crate::UriError;

#[derive(Debug, Clone)]
pub struct DexieCat {
    pub hash: Bytes32,
    pub name: Option<String>,
    pub icon_url: Option<String>,
    pub description: Option<String>,
    pub ticker: Option<String>,
    pub hidden_puzzle_hash: Option<Bytes32>,
}

impl DexieCat {
    pub async fn fetch_all(testnet: bool) -> Result<Vec<Self>, UriError> {
        let mut page = 1;
        let mut assets = Vec::new();

        loop {
            let response = reqwest::get(format!(
                "{}/assets?page_size=100&page={page}&type=cat",
                dexie_base_url(testnet)
            ))
            .await?
            .json::<AssetResponse>()
            .await?;

            if response.assets.is_empty() {
                break;
            }

            for asset in response.assets {
                assets.push(Self {
                    hash: asset.id,
                    name: asset.name,
                    icon_url: Some(format!(
                        "{}/{}.webp",
                        dexie_image_base_url(testnet),
                        asset.id
                    )),
                    description: asset.description,
                    ticker: asset.code,
                    hidden_puzzle_hash: asset.hidden_puzzle_hash,
                });
            }

            page += 1;
        }

        Ok(assets)
    }

    pub async fn fetch(asset_id: Bytes32, testnet: bool) -> Result<Self, UriError> {
        let response = reqwest::get(format!(
            "{}/assets?page_size=25&page=1&type=cat&code={asset_id}",
            dexie_base_url(testnet)
        ))
        .await?
        .json::<AssetResponse>()
        .await?;

        let asset = response.assets.first().cloned().unwrap_or_default();
        let icon_url = (!response.assets.is_empty()).then_some(format!(
            "{}/{}.webp",
            dexie_image_base_url(testnet),
            asset.id
        ));

        Ok(Self {
            hash: asset_id,
            name: asset.name,
            icon_url,
            description: asset.description,
            ticker: asset.code,
            hidden_puzzle_hash: asset.hidden_puzzle_hash,
        })
    }
}

#[derive(Deserialize)]
struct AssetResponse {
    assets: Vec<AssetData>,
}

#[derive(Deserialize, Default, Clone)]
struct AssetData {
    id: Bytes32,
    name: Option<String>,
    code: Option<String>,
    description: Option<String>,
    #[serde(default)]
    hidden_puzzle_hash: Option<Bytes32>,
}

fn dexie_base_url(testnet: bool) -> &'static str {
    if testnet {
        "https://api-testnet.dexie.space/v1"
    } else {
        "https://api.dexie.space/v1"
    }
}

fn dexie_image_base_url(testnet: bool) -> &'static str {
    if testnet {
        "https://icons-testnet.dexie.space"
    } else {
        "https://icons.dexie.space"
    }
}

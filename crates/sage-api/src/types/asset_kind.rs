use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AssetKind {
    Unknown,
    Launcher,
    Token {
        asset_id: String,
        name: Option<String>,
        icon_url: Option<String>,
        ticker: Option<String>,
        precision: u8,
        is_xch: bool,
    },
    Did {
        asset_id: String,
        name: Option<String>,
        icon_url: Option<String>,
    },
    Nft {
        asset_id: String,
        name: Option<String>,
        icon_url: Option<String>,
    },
    Option, // TODO: add option fields
}

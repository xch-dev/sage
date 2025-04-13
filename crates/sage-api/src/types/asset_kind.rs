use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AssetKind {
    Unknown,
    Xch,
    Launcher,
    Cat {
        asset_id: String,
        name: Option<String>,
        ticker: Option<String>,
        icon_url: Option<String>,
    },
    Did {
        launcher_id: String,
        name: Option<String>,
    },
    Nft {
        launcher_id: String,
        icon: Option<String>,
        name: Option<String>,
    },
}

use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
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
        image_data: Option<String>,
        image_mime_type: Option<String>,
        name: Option<String>,
    },
}

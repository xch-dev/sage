use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct Asset {
    pub asset_id: Option<String>,
    pub name: Option<String>,
    pub ticker: Option<String>,
    pub precision: u8,
    pub icon_url: Option<String>,
    pub description: Option<String>,
    pub is_sensitive_content: bool,
    pub is_visible: bool,
    pub revocation_address: Option<String>,
    pub kind: AssetKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum AssetKind {
    Token,
    Nft,
    Did,
    Option,
    Vault,
}

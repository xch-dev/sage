use serde::{Deserialize, Serialize};

use crate::Amount;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct FeePolicy {
    /// Address receiving transfer fees
    pub recipient: String,
    /// Transfer fee in basis points (1/100 of a percent)
    pub fee_basis_points: u16,
    /// Minimum fee amount in mojos
    pub min_fee: Amount,
    /// Whether zero-price transfers can bypass fees
    pub allow_zero_price: bool,
    /// Whether revocations can bypass fees
    pub allow_revoke_fee_bypass: bool,
}

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
    pub fee_policy: Option<FeePolicy>,
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
}

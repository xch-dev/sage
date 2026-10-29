use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct WalletInfo {
    pub name: String,
    pub fingerprint: u32,
    pub kind: WalletKind,
    pub network_id: String,
    pub emoji: Option<String>,
}

/// The kind of wallet, which determines how its addresses are found and whether it can sign
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum WalletKind {
    /// A BLS key, whose addresses are derived from its master public key. The key is in the
    /// keychain under the wallet's fingerprint
    Bls,
}

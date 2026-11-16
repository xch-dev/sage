use serde::{Deserialize, Serialize};

/// One of the wallet's addresses. A BLS wallet's addresses are derived from its key, and a
/// vault's are its P2 singleton addresses, numbered by their nonce
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DerivationRecord {
    /// The derivation index of a BLS wallet's address, or the nonce of a vault's address
    pub index: u32,
    /// The synthetic public key of a BLS wallet's address, in hex. A vault's addresses aren't
    /// locked to a key, so they have none
    pub public_key: Option<String>,
    pub address: String,
}

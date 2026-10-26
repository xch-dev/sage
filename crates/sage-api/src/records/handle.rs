use serde::{Deserialize, Serialize};

/// A handle mapping which was verified against the chain
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct HandleRecord {
    /// The handle, without the leading `@`
    #[cfg_attr(feature = "openapi", schema(example = "alice"))]
    pub handle: String,
    /// The hash the registry stores the handle under
    pub handle_hash: String,
    /// The address the handle resolves to
    pub address: String,
    /// Whether the resolved address is new since it was last verified
    pub status: HandleStatus,
    /// The address the handle resolved to before it changed, while the change is
    /// unacknowledged
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub previous_address: Option<String>,
    /// The address this handle last resolved to when it was sent to
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub last_sent_address: Option<String>,
    /// Whether the mapping comes from an earlier verification, because the handle could
    /// not be verified right now
    pub cached: bool,
    /// Launcher ID of the singleton which owns the handle
    pub owner_launcher_id: String,
    /// Launcher ID of the singleton the handle resolves to
    pub resolved_launcher_id: String,
    /// Unix timestamp after which the handle expires
    pub expiration: u64,
    /// The handle's registry slot coin
    pub slot_coin_id: String,
    /// Height the slot coin was confirmed at
    pub slot_confirmation_height: u32,
    /// The resolved singleton coin the address was derived from
    pub resolved_coin_id: String,
    /// Height of the peak transaction block the mapping was verified at
    pub verified_height: u32,
    /// Timestamp of the peak transaction block the mapping was verified at
    pub verified_timestamp: u64,
    /// Local Unix timestamp of the verification
    pub last_verified_at: u64,
}

/// Whether a verified handle mapping has changed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum HandleStatus {
    /// The handle resolves to the same address as when it was first verified, or the
    /// change has been acknowledged
    Verified,
    /// The handle now resolves to a different address than it did previously
    Changed,
}

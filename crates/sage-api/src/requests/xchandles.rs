use serde::{Deserialize, Serialize};

use crate::{
    AddressHandlesRecord, HandleFilterMode, HandleRecord, HandleSlotRecord, PendingTransferRecord,
    PendingTransferStatus,
};

/// Resolve an XCHandles handle to an address
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "XCHandles",
        description = "Resolve an XCHandles handle to the address it points to. The result is verified against the blockchain through connected peers before it is returned. If the handle cannot be verified right now, a mapping verified within the last day may be returned instead, marked as cached."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ResolveHandle {
    /// The handle, with or without a leading `@`
    #[cfg_attr(feature = "openapi", schema(example = "@alice"))]
    pub handle: String,
}

/// Response with the verified handle mapping
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "XCHandles"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ResolveHandleResponse {
    /// The verified handle mapping
    pub handle: HandleRecord,
}

/// Get the full verified details of an XCHandles handle
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "XCHandles",
        description = "Resolve an XCHandles handle like resolve_handle, and also return the registry slot it was verified against and whether a transfer of the handle is pending. A pending transfer reported by the API is checked against the blockchain before it is marked verified."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetHandle {
    /// The handle, with or without a leading `@`
    #[cfg_attr(feature = "openapi", schema(example = "@alice"))]
    pub handle: String,
}

/// Response with the full verified details of a handle
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "XCHandles"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetHandleResponse {
    /// The verified handle mapping
    pub handle: HandleRecord,
    /// The registry slot the mapping was verified against
    pub slot: HandleSlotRecord,
    /// Whether a transfer of the handle is pending
    pub pending_transfer_status: PendingTransferStatus,
    /// The pending transfer reported by the API, unless the status is none or unavailable
    #[cfg_attr(feature = "openapi", schema(nullable = true))]
    pub pending_transfer: Option<PendingTransferRecord>,
}

/// List the handles Sage knows about
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "XCHandles",
        description = "List handle mappings Sage has already verified, from the local store. Nothing is fetched from the network unless refresh is set, in which case handle NFTs owned by this wallet are verified first, using their display names only as hints."
    )
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetHandles {
    /// Which handles to list
    #[serde(default)]
    pub filter: HandleFilterMode,
    /// Also list mappings which are unverified, stale or expired
    #[serde(default)]
    pub include_unverified: bool,
    /// Verify the handles named by NFTs in this wallet before listing. This makes requests
    /// to the XCHandles API and peers
    #[serde(default)]
    pub refresh: bool,
    /// Starting offset for pagination
    #[cfg_attr(feature = "openapi", schema(example = 0))]
    pub offset: u32,
    /// Number of handles to return
    #[cfg_attr(feature = "openapi", schema(example = 50))]
    pub limit: u32,
}

/// Response with known handles
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "XCHandles"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GetHandlesResponse {
    /// The handles, sorted by handle
    pub handles: Vec<HandleRecord>,
    /// Total number of handles matching the filter
    pub total: u32,
}

/// Look up the known handles for addresses
#[cfg_attr(
    feature = "openapi",
    crate::openapi_attr(
        tag = "XCHandles",
        description = "Find the verified handles Sage already knows which resolve to each of the given addresses or puzzle hashes. The XCHandles API has no reverse lookup, so this only uses the local store and never makes network requests. Up to 1000 addresses can be looked up at once."
    )
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct LookupAddressHandles {
    /// Addresses on the current network, or hex encoded puzzle hashes
    pub addresses: Vec<String>,
}

/// Response with the known handles for each address
#[cfg_attr(feature = "openapi", crate::openapi_attr(tag = "XCHandles"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct LookupAddressHandlesResponse {
    /// One entry per requested address, in the same order
    pub results: Vec<AddressHandlesRecord>,
}

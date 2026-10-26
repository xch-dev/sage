use serde::{Deserialize, Serialize};

use crate::HandleRecord;

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

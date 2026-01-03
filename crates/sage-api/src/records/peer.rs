use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "tauri", derive(specta::Type))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PeerRecord {
    pub ip_addr: String,
    pub port: u16,
    pub peak_height: u32,
    pub user_managed: bool,
}

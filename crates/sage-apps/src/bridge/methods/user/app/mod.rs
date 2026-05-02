pub mod events;
pub mod get_capabilities;
pub mod get_info;
pub mod lifecycle;
pub mod request_capability_grant;
pub mod request_network_whitelist_grant;

use std::path::PathBuf;

use tauri::Manager;

pub use events::{GrantedCapabilitiesChangeEvent, GrantedNetworkWhitelistChangeEvent};
pub use get_capabilities::AppGetCapabilities;
pub use get_info::AppGetInfo;
pub use lifecycle::{AppLifecycleReadyToStop, AppLifecycleSetBeforeStopListener};
pub use request_capability_grant::AppRequestCapabilityGrant;
pub use request_network_whitelist_grant::AppRequestNetworkWhitelistGrant;

use crate::bridge::methods::BridgeTools;
use crate::bridge::methods::shared::BridgeMethodHandleError;

pub(crate) fn resolve_app_base_path(tools: &BridgeTools<'_>) -> Result<PathBuf, BridgeMethodHandleError> {
    tools.app_handle.path().app_data_dir().map_err(|err| {
        BridgeMethodHandleError::internal_error(format!("failed to resolve app data dir: {err}"))
    })
}

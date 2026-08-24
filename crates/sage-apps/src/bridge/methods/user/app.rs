mod events;
mod get_capabilities;
mod get_info;
mod lifecycle;
mod request_capability_grant;
mod request_network_whitelist_grant;
mod request_permission_grants;

pub(crate) use events::*;
pub(crate) use get_capabilities::*;
pub(crate) use get_info::*;
pub(crate) use lifecycle::*;
pub(crate) use request_capability_grant::*;
pub(crate) use request_network_whitelist_grant::*;
pub(crate) use request_permission_grants::*;

use std::path::PathBuf;

use tauri::Manager;

use crate::{BridgeMethodHandleError, BridgeTools};

pub(crate) fn resolve_app_base_path(
    tools: &BridgeTools<'_>,
) -> Result<PathBuf, BridgeMethodHandleError> {
    tools.app_handle.path().app_data_dir().map_err(|err| {
        BridgeMethodHandleError::internal_error(format!("failed to resolve app data dir: {err}"))
    })
}

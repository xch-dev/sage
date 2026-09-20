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

use crate::BridgeTools;

pub(crate) fn resolve_app_base_path(tools: &BridgeTools<'_>) -> PathBuf {
    tools.host_state.root.clone()
}

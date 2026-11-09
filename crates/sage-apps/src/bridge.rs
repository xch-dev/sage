mod bridge_request;
mod commands;
mod debug;
mod event_emit;
mod methods;
mod registry;
mod state;
mod ts_exports;
mod types;

use tauri::{AppHandle, State};

use crate::AppsHostState;

pub use commands::*;
pub use ts_exports::*;
pub use types::*;

pub(crate) use bridge_request::*;
pub(crate) use debug::*;
pub(crate) use event_emit::*;
pub(crate) use methods::*;
pub(crate) use registry::*;
pub(crate) use state::*;

pub async fn emit_selected_wallet_changed(
    app_handle: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    fingerprint: u32,
) {
    emit_selected_wallet_changed_inner(app_handle, apps_state, fingerprint).await;
}

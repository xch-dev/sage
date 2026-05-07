use std::path::PathBuf;

use tauri::{AppHandle, Manager};

use crate::bridge::methods::system::emit_listed_apps_changed;
use crate::host::AppState;
use crate::types::{SageAppWalletScope, SharedSageApp};

pub async fn update_app_wallet_scope_for_app(
    app_handle: &AppHandle,
    app: &SharedSageApp,
    wallet_scope: SageAppWalletScope,
) -> anyhow::Result<()> {
    app.try_mutate(|sage_app| {
        sage_app.common_mut().update_wallet_scope(wallet_scope);
        Ok::<_, anyhow::Error>(())
    })
        .map_err(|err| anyhow::anyhow!(err))?;

    let base_path: PathBuf = {
        let state = app_handle.state::<AppState>();
        let state = state.lock().await;
        state.path.clone()
    };

    let apps_state = app_handle.state::<crate::AppsHostState>();
    emit_listed_apps_changed(app_handle, &apps_state, &base_path).await;

    Ok(())
}

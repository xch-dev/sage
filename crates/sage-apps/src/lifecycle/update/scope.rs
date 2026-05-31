use tauri::{AppHandle, Manager};

use crate::bridge::emit_listed_apps_changed;
use crate::types::{SageAppWalletScope, SharedSageApp};

pub async fn update_app_wallet_scope_for_app(
    app_handle: &AppHandle,
    app: &SharedSageApp,
    wallet_scope: SageAppWalletScope,
) -> anyhow::Result<()> {
    let apps_state = app_handle.state::<crate::AppsHostState>();
    let manager = crate::lifecycle::AppMutationManager::new(app_handle, &apps_state);

    manager
        .mutate_shared_app(app, move |ctx| {
            Box::pin(async move {
                ctx.draft_mut()
                    .app_mut()
                    .common_mut()
                    .update_wallet_scope(wallet_scope);

                Ok(())
            })
        })
        .await
        .map_err(anyhow::Error::msg)?;

    emit_listed_apps_changed(app_handle, &apps_state).await;

    Ok(())
}

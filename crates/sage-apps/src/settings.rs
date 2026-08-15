use tauri::{State, command};

use crate::{AppsHostState, Result};

#[command]
#[specta::specta]
pub async fn apps_get_auto_update_enabled(apps_state: State<'_, AppsHostState>) -> Result<bool> {
    Ok(apps_state.db.get_auto_update_enabled().await?)
}

#[command]
#[specta::specta]
pub async fn apps_set_auto_update_enabled(
    apps_state: State<'_, AppsHostState>,
    enabled: bool,
) -> Result<bool> {
    Ok(apps_state.db.set_auto_update_enabled(enabled).await?)
}

use specta::specta;
use tauri::{command, State};

use crate::{app_state::AppState, error::Result};

#[command]
#[specta]
pub async fn initialize(state: State<'_, AppState>) -> Result<()> {
    state.lock().await.initialize().await
}

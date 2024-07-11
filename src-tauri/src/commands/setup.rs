use tauri::{command, State};

use crate::{app_state::AppState, error::Result};

#[command]
pub async fn initialize(state: State<'_, AppState>) -> Result<()> {
    let mut state = state.lock().await;
    state.initialize().await
}

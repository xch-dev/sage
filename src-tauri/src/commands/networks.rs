use indexmap::IndexMap;
use sage::Network;
use tauri::{command, State};

use crate::{app_state::AppState, error::Result};

#[command]
pub async fn network_list(state: State<'_, AppState>) -> Result<IndexMap<String, Network>> {
    let state = state.lock().await;
    Ok(state.networks().clone())
}

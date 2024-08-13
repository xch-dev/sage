use tauri::{command, State};

use crate::{app_state::AppState, error::Result, models::PeerInfo};

#[command]
pub async fn peer_list(state: State<'_, AppState>) -> Result<Vec<PeerInfo>> {
    let state = state.lock().await;
    let sync_manager = state.sync_manager.lock().await;

    Ok(sync_manager
        .peers()
        .map(|peer| PeerInfo {
            ip_addr: peer.socket_addr().ip().to_string(),
            port: peer.socket_addr().port(),
            node_id: *peer.peer_id().as_bytes(),
            trusted: false,
        })
        .collect())
}

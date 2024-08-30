use std::{net::IpAddr, str::FromStr};

use itertools::Itertools;
use tauri::{command, State};

use crate::{app_state::AppState, error::Result, models::PeerInfo};

#[command]
pub async fn peer_list(state: State<'_, AppState>) -> Result<Vec<PeerInfo>> {
    let state = state.lock().await;
    let sync_state = state.sync_state.lock().await;

    Ok(sync_state
        .peers()
        .sorted_by_key(|peer| peer.socket_addr().ip())
        .map(|peer| PeerInfo {
            ip_addr: peer.socket_addr().ip().to_string(),
            port: peer.socket_addr().port(),
            trusted: false,
        })
        .collect())
}

#[command]
pub async fn remove_peer(state: State<'_, AppState>, ip_addr: String, ban: bool) -> Result<()> {
    let state = state.lock().await;
    let mut sync_state = state.sync_state.lock().await;

    let ip_addr = IpAddr::from_str(&ip_addr)?;

    if ban {
        sync_state.ban(ip_addr);
    } else {
        sync_state.remove_peer(ip_addr);
    }

    Ok(())
}

use std::{net::IpAddr, str::FromStr};

use itertools::Itertools;
use specta::specta;
use tauri::{command, State};

use crate::{app_state::AppState, error::Result, models::PeerInfo};

#[command]
#[specta]
pub async fn peer_list(state: State<'_, AppState>) -> Result<Vec<PeerInfo>> {
    let state = state.lock().await;
    let peer_state = state.peer_state.lock().await;

    Ok(peer_state
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
#[specta]
pub async fn remove_peer(state: State<'_, AppState>, ip_addr: String, ban: bool) -> Result<()> {
    let state = state.lock().await;
    let mut peer_state = state.peer_state.lock().await;

    let ip_addr = IpAddr::from_str(&ip_addr)?;

    if ban {
        peer_state.ban(ip_addr);
    } else {
        peer_state.remove_peer(ip_addr);
    }

    Ok(())
}

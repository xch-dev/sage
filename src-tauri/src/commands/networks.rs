use indexmap::IndexMap;
use specta::specta;
use tauri::{command, State};

use crate::{app_state::AppState, error::Result, models::NetworkInfo};

#[command]
#[specta]
pub async fn network_list(state: State<'_, AppState>) -> Result<IndexMap<String, NetworkInfo>> {
    let state = state.lock().await;

    let mut networks = IndexMap::new();

    for (network_id, network) in &state.networks {
        let info = NetworkInfo {
            default_port: network.default_port,
            genesis_challenge: hex::encode(network.genesis_challenge),
            agg_sig_me: network.agg_sig_me.map(hex::encode),
            dns_introducers: network.dns_introducers.clone(),
        };
        networks.insert(network_id.clone(), info);
    }

    Ok(networks)
}

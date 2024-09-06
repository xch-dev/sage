use std::net::IpAddr;

use chia_wallet_sdk::NetworkId;
use indexmap::IndexMap;
use itertools::Itertools;
use sage_api::PeerRecord;
use sage_config::{NetworkConfig, WalletConfig};
use sage_wallet::SyncCommand;
use specta::specta;
use tauri::{command, State};

use crate::{app_state::AppState, error::Result, models::NetworkInfo};

#[command]
#[specta]
pub async fn get_peers(state: State<'_, AppState>) -> Result<Vec<PeerRecord>> {
    let state = state.lock().await;
    let peer_state = state.peer_state.lock().await;

    Ok(peer_state
        .peers()
        .sorted_by_key(|info| info.peer.socket_addr().ip())
        .map(|info| PeerRecord {
            ip_addr: info.peer.socket_addr().ip().to_string(),
            port: info.peer.socket_addr().port(),
            trusted: false,
            peak_height: info.claimed_peak,
        })
        .collect())
}

#[command]
#[specta]
pub async fn remove_peer(state: State<'_, AppState>, ip_addr: IpAddr, ban: bool) -> Result<()> {
    let state = state.lock().await;
    let mut peer_state = state.peer_state.lock().await;

    if ban {
        peer_state.ban(ip_addr);
    } else {
        peer_state.remove_peer(ip_addr);
    }

    Ok(())
}

#[command]
#[specta]
pub async fn add_peer(state: State<'_, AppState>, ip: IpAddr, trusted: bool) -> Result<()> {
    state
        .lock()
        .await
        .command_sender
        .send(SyncCommand::ConnectPeer { ip, trusted })
        .await?;

    Ok(())
}

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

#[command]
#[specta]
pub async fn network_config(state: State<'_, AppState>) -> Result<NetworkConfig> {
    let state = state.lock().await;
    Ok(state.config.network.clone())
}

#[command]
#[specta]
pub async fn set_discover_peers(state: State<'_, AppState>, discover_peers: bool) -> Result<()> {
    let mut state = state.lock().await;

    if state.config.network.discover_peers != discover_peers {
        state.config.network.discover_peers = discover_peers;
        state.save_config()?;
        state
            .command_sender
            .send(SyncCommand::SetDiscoverPeers(discover_peers))
            .await?;
    }

    Ok(())
}

#[command]
#[specta]
pub async fn set_target_peers(state: State<'_, AppState>, target_peers: u32) -> Result<()> {
    let mut state = state.lock().await;

    state.config.network.target_peers = target_peers;
    state.save_config()?;
    state
        .command_sender
        .send(SyncCommand::SetTargetPeers(target_peers as usize))
        .await?;

    Ok(())
}

#[command]
#[specta]
pub async fn set_network_id(state: State<'_, AppState>, network_id: String) -> Result<()> {
    let mut state = state.lock().await;

    state.config.network.network_id.clone_from(&network_id);
    state.save_config()?;

    state
        .command_sender
        .send(SyncCommand::SwitchNetwork {
            network_id: if network_id == "mainnet" {
                NetworkId::Mainnet
            } else {
                NetworkId::Testnet11
            },
            network: state.networks[&network_id].clone(),
        })
        .await?;

    state.switch_wallet().await?;

    Ok(())
}

#[command]
#[specta]
pub async fn wallet_config(state: State<'_, AppState>, fingerprint: u32) -> Result<WalletConfig> {
    let state = state.lock().await;
    state.try_wallet_config(fingerprint).cloned()
}

#[command]
#[specta]
pub async fn set_derive_automatically(
    state: State<'_, AppState>,
    fingerprint: u32,
    derive_automatically: bool,
) -> Result<()> {
    let mut state = state.lock().await;

    let config = state.try_wallet_config_mut(fingerprint)?;

    if config.derive_automatically != derive_automatically {
        config.derive_automatically = derive_automatically;
        state.save_config()?;
    }

    Ok(())
}

#[command]
#[specta]
pub async fn set_derivation_batch_size(
    state: State<'_, AppState>,
    fingerprint: u32,
    derivation_batch_size: u32,
) -> Result<()> {
    let mut state = state.lock().await;

    let config = state.try_wallet_config_mut(fingerprint)?;
    config.derivation_batch_size = derivation_batch_size;
    state.save_config()?;

    // TODO: Update sync manager

    Ok(())
}

#[command]
#[specta]
pub async fn rename_wallet(
    state: State<'_, AppState>,
    fingerprint: u32,
    name: String,
) -> Result<()> {
    let mut state = state.lock().await;

    let config = state.try_wallet_config_mut(fingerprint)?;
    config.name = name;
    state.save_config()?;

    Ok(())
}

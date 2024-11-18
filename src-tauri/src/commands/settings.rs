use std::{net::IpAddr, time::Duration};

use indexmap::IndexMap;
use itertools::Itertools;
use sage_api::PeerRecord;
use sage_config::{Network, NetworkConfig, WalletConfig};
use sage_wallet::SyncCommand;
use specta::specta;
use tauri::{command, State};

use crate::{app_state::AppState, error::Result, parse::parse_genesis_challenge};

#[command]
#[specta]
pub async fn get_peers(state: State<'_, AppState>) -> Result<Vec<PeerRecord>> {
    let state = state.lock().await;
    let peer_state = state.peer_state.lock().await;

    Ok(peer_state
        .peers_with_heights()
        .into_iter()
        .sorted_by_key(|info| info.0.socket_addr().ip())
        .map(|info| PeerRecord {
            ip_addr: info.0.socket_addr().ip().to_string(),
            port: info.0.socket_addr().port(),
            trusted: false,
            peak_height: info.1,
        })
        .collect())
}

#[command]
#[specta]
pub async fn remove_peer(state: State<'_, AppState>, ip_addr: IpAddr, ban: bool) -> Result<()> {
    let state = state.lock().await;
    let mut peer_state = state.peer_state.lock().await;

    if ban {
        peer_state.ban(ip_addr, Duration::from_secs(60 * 60), "manually banned");
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
pub async fn network_list(state: State<'_, AppState>) -> Result<IndexMap<String, Network>> {
    let state = state.lock().await;
    Ok(state.networks.clone())
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
            .send(SyncCommand::SetTargetPeers(if discover_peers {
                state.config.network.target_peers as usize
            } else {
                0
            }))
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

    let network = state.network();

    state
        .command_sender
        .send(SyncCommand::SwitchNetwork {
            network_id,
            network: chia_wallet_sdk::Network {
                default_port: network.default_port,
                genesis_challenge: parse_genesis_challenge(network.genesis_challenge.clone())?,
                dns_introducers: network.dns_introducers.clone(),
            },
        })
        .await?;

    state.switch_wallet().await?;

    Ok(())
}

#[command]
#[specta]
pub async fn wallet_config(state: State<'_, AppState>, fingerprint: u32) -> Result<WalletConfig> {
    let state = state.lock().await;
    Ok(state.try_wallet_config(fingerprint).cloned()?)
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

use sage_config::{NetworkConfig, WalletConfig};
use specta::specta;
use tauri::{command, State};

use crate::{app_state::AppState, error::Result};

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
        state.reset_sync_task(false)?;
    }

    Ok(())
}

#[command]
#[specta]
pub async fn set_target_peers(state: State<'_, AppState>, target_peers: u32) -> Result<()> {
    let mut state = state.lock().await;

    state.config.network.target_peers = target_peers;
    state.save_config()?;
    state.reset_sync_task(false)?;

    Ok(())
}

#[command]
#[specta]
pub async fn set_network_id(state: State<'_, AppState>, network_id: String) -> Result<()> {
    let mut state = state.lock().await;

    state.config.network.network_id = network_id;
    state.save_config()?;
    state.reset_sync_task(true)?;
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

    // TODO: Only if needed.
    state.reset_sync_task(false)?;

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

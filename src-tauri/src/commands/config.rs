use tauri::{command, State};

use crate::{
    app_state::AppState,
    config::{DerivationMode, NetworkConfig, PeerMode, WalletConfig},
    error::Result,
};

#[command]
pub async fn network_config(state: State<'_, AppState>) -> Result<NetworkConfig> {
    let state = state.lock().await;
    Ok(state.config.network.clone())
}

#[command]
pub async fn set_peer_mode(state: State<'_, AppState>, peer_mode: PeerMode) -> Result<()> {
    let mut state = state.lock().await;

    state.config.network.peer_mode = peer_mode;
    state.save_config()?;
    state.reset_sync_task(false).await?;

    Ok(())
}

#[command]
pub async fn set_target_peers(state: State<'_, AppState>, target_peers: usize) -> Result<()> {
    let mut state = state.lock().await;

    state.config.network.target_peers = target_peers;
    state.save_config()?;
    state.reset_sync_task(false).await?;

    Ok(())
}

#[command]
pub async fn set_network_id(state: State<'_, AppState>, network_id: String) -> Result<()> {
    let mut state = state.lock().await;

    state.config.network.network_id = network_id;
    state.save_config()?;
    state.reset_sync_task(true).await?;

    Ok(())
}

#[command]
pub async fn wallet_config(state: State<'_, AppState>, fingerprint: u32) -> Result<WalletConfig> {
    let state = state.lock().await;
    state.try_wallet_config(fingerprint).cloned()
}

#[command]
pub async fn set_derivation_mode(
    state: State<'_, AppState>,
    fingerprint: u32,
    derivation_mode: DerivationMode,
) -> Result<()> {
    let mut state = state.lock().await;

    let config = state.try_wallet_config_mut(fingerprint)?;
    config.derivation_mode = derivation_mode;
    state.save_config()?;

    Ok(())
}

#[command]
pub async fn set_derivation_batch_size(
    state: State<'_, AppState>,
    fingerprint: u32,
    derivation_batch_size: u32,
) -> Result<()> {
    let mut state = state.lock().await;

    let config = state.try_wallet_config_mut(fingerprint)?;
    config.derivation_batch_size = derivation_batch_size;
    state.save_config()?;

    if let Some(wallet) = state.wallet.as_ref() {
        if wallet.fingerprint() == fingerprint {
            // TODO: wallet.initial_sync(derivation_batch_size).await?;
        }
    }

    Ok(())
}

#[command]
pub async fn set_derivation_index(
    state: State<'_, AppState>,
    fingerprint: u32,
    derivation_index: u32,
) -> Result<()> {
    let mut state = state.lock().await;

    let config = state.try_wallet_config_mut(fingerprint)?;
    config.derivation_index = derivation_index;
    state.save_config()?;

    Ok(())
}

#[command]
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

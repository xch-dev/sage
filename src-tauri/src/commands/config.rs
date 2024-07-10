use tauri::{command, State};

use crate::{
    app_state::AppState,
    config::{DerivationMode, WalletConfig},
    error::Result,
};

#[command]
pub async fn wallet_config(state: State<'_, AppState>, fingerprint: u32) -> Result<WalletConfig> {
    let state = state.lock().await;
    state.wallet_config(fingerprint)
}

#[command]
pub async fn set_derivation_mode(
    state: State<'_, AppState>,
    fingerprint: u32,
    derivation_mode: DerivationMode,
) -> Result<()> {
    let state = state.lock().await;
    state.update_wallet_config(fingerprint, |config| {
        config.derivation_mode = derivation_mode;
    })?;
    Ok(())
}

#[command]
pub async fn set_derivation_batch_size(
    state: State<'_, AppState>,
    fingerprint: u32,
    derivation_batch_size: u32,
) -> Result<()> {
    let state = state.lock().await;
    state.update_wallet_config(fingerprint, |config| {
        config.derivation_batch_size = derivation_batch_size;
    })?;
    Ok(())
}

#[command]
pub async fn rename_wallet(
    state: State<'_, AppState>,
    fingerprint: u32,
    name: String,
) -> Result<()> {
    let state = state.lock().await;
    state.update_wallet_config(fingerprint, |config| {
        config.name = name;
    })?;
    Ok(())
}

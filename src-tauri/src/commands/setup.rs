use std::fs;

use tauri::{command, State};

use crate::{app_state::AppState, error::Result};

#[command]
pub async fn initialize(state: State<'_, AppState>) -> Result<()> {
    let mut state = state.lock().await;

    fs::create_dir_all(&state.path)?;

    let key_path = state.path.join("keys.bin");
    let config_path = state.path.join("config.toml");
    let networks_path = state.path.join("networks.toml");

    if !key_path.try_exists()? {
        fs::write(&key_path, bincode::serialize(&state.keys)?)?;
    } else {
        let data = fs::read(&key_path)?;
        state.keys = bincode::deserialize(&data)?;
    }

    if !config_path.try_exists()? {
        fs::write(&config_path, toml::to_string_pretty(&state.config)?)?;
    } else {
        let text = fs::read_to_string(&config_path)?;
        state.config = toml::from_str(&text)?;
    };

    if !networks_path.try_exists()? {
        fs::write(&networks_path, toml::to_string_pretty(&state.networks)?)?;
    } else {
        let text = fs::read_to_string(&networks_path)?;
        state.networks = toml::from_str(&text)?;
    }

    if let Some(fingerprint) = state.config.wallet.active_fingerprint {
        state.login_wallet(fingerprint).await?;
    }

    state.start_peer_discovery()?;

    Ok(())
}

use std::fs;

use tauri::{command, State};
use tracing::{info, level_filters::LevelFilter};
use tracing_appender::rolling::{Builder, Rotation};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};

use crate::{app_state::AppState, error::Result};

#[command]
pub async fn initialize(state: State<'_, AppState>) -> Result<()> {
    let mut state = state.lock().await;

    fs::create_dir_all(&state.path)?;

    let key_path = state.path.join("keys.bin");
    let config_path = state.path.join("config.toml");
    let networks_path = state.path.join("networks.toml");

    let log_dir = state.path.join("log");
    if !log_dir.exists() {
        std::fs::create_dir_all(&log_dir)?;
    }

    if key_path.try_exists()? {
        let data = fs::read(&key_path)?;
        state.keys = bincode::deserialize(&data)?;
    } else {
        fs::write(&key_path, bincode::serialize(&state.keys)?)?;
    }

    if config_path.try_exists()? {
        let text = fs::read_to_string(&config_path)?;
        state.config = toml::from_str(&text)?;
    } else {
        fs::write(&config_path, toml::to_string_pretty(&state.config)?)?;
    };

    if networks_path.try_exists()? {
        let text = fs::read_to_string(&networks_path)?;
        state.networks = toml::from_str(&text)?;
    } else {
        fs::write(&networks_path, toml::to_string_pretty(&state.networks)?)?;
    }

    let log_level = state.config.app.log_level.parse()?;

    let log_file = Builder::new()
        .filename_prefix("app.log")
        .rotation(Rotation::DAILY)
        .max_log_files(3)
        .build(log_dir.as_path())?;

    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(log_file)
        .with_ansi(false);

    let stdout_layer = tracing_subscriber::fmt::layer().pretty();

    tracing_subscriber::registry()
        .with(stdout_layer.with_filter(LevelFilter::from_level(log_level)))
        .with(file_layer.with_filter(LevelFilter::from_level(log_level)))
        .init();

    info!("Initial setup complete");

    state.switch_wallet().await?;

    Ok(())
}

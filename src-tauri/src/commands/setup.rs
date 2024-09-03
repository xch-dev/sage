use std::fs;

use chia_wallet_sdk::Network;
use indexmap::IndexMap;
use specta::specta;
use tauri::{command, State};
use tracing::{info, level_filters::LevelFilter};
use tracing_appender::rolling::{Builder, Rotation};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};

use crate::{app_state::AppState, error::Result, models::NetworkInfo};

#[command]
#[specta]
pub async fn initialize(state: State<'_, AppState>) -> Result<()> {
    let mut state = state.lock().await;

    if state.initialized {
        return Ok(());
    }

    state.initialized = true;

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
        let networks: IndexMap<String, NetworkInfo> = toml::from_str(&text)?;

        for (network_id, network) in networks {
            state.networks.insert(
                network_id,
                Network {
                    default_port: network.default_port,
                    genesis_challenge: hex::decode(&network.genesis_challenge)?.try_into()?,
                    agg_sig_me: network
                        .agg_sig_me
                        .map(|x| Result::Ok(hex::decode(&x)?.try_into()?))
                        .transpose()?,
                    dns_introducers: network.dns_introducers,
                },
            );
        }
    } else {
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

        fs::write(&networks_path, toml::to_string_pretty(&networks)?)?;
    }

    let log_level = state.config.app.log_level.parse()?;

    let log_file = Builder::new()
        .filename_prefix("app.log")
        .rotation(Rotation::DAILY)
        .max_log_files(3)
        .build(log_dir.as_path())?;

    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(log_file)
        .with_ansi(false)
        .with_target(false)
        .compact();

    // TODO: Fix ANSI better
    #[cfg(not(mobile))]
    let stdout_layer = tracing_subscriber::fmt::layer().pretty();

    let registry = tracing_subscriber::registry()
        .with(file_layer.with_filter(LevelFilter::from_level(log_level)));

    #[cfg(not(mobile))]
    {
        registry
            .with(stdout_layer.with_filter(LevelFilter::from_level(log_level)))
            .init();
    }

    #[cfg(mobile)]
    registry.init();

    info!("Initial setup complete");

    state.switch_wallet().await?;

    Ok(())
}

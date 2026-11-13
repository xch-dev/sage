//! A throwaway Sage backend on a simulated chain, for testing the UI in a browser.
//!
//! This starts a `PeerSimulator` and two Sage backends connected to it, each with its own funded
//! wallet, then serves both over plain HTTP in place of Tauri's IPC. `pnpm sandbox` runs it next to
//! the Vite dev server. See `docs/sandbox.md`.

mod bridge;
mod endpoints;
mod error;
mod events;
mod wallet;

#[cfg(test)]
mod tests;

pub use bridge::*;
pub use endpoints::*;
pub use error::*;
pub use events::*;
pub use wallet::*;

use std::{fs, net::SocketAddr, path::PathBuf};

use anyhow::{Context, Result};
use chia_wallet_sdk::test::PeerSimulator;
use clap::Parser;
use rustls::crypto::aws_lc_rs::default_provider;
use serde::Serialize;
use tokio::net::TcpListener;
use tracing::info;

#[derive(Debug, Parser)]
struct Cli {
    /// Directory for the wallets' data and the ready file. Its `data` folder is wiped on startup.
    #[arg(long)]
    dir: PathBuf,

    /// Port for the bridge, or 0 to pick a free one.
    #[arg(long, default_value_t = 0)]
    port: u16,
}

/// Written to `backend.json` once every wallet has synced, so that the launcher knows it's ready.
#[derive(Debug, Serialize)]
struct Ready {
    port: u16,
    wallets: Vec<WalletInfo>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let _ = default_provider().install_default();

    let ready_path = cli.dir.join("backend.json");
    let data_dir = cli.dir.join("data");

    if ready_path.try_exists()? {
        fs::remove_file(&ready_path)?;
    }

    if data_dir.try_exists()? {
        fs::remove_dir_all(&data_dir).context("failed to clear the previous sandbox data")?;
    }

    fs::create_dir_all(&data_dir)?;

    // Bind first, so that the wallets can be pointed at the bridge's XCHandles stand-in.
    let listener = TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], cli.port)))
        .await
        .with_context(|| format!("failed to bind the bridge to port {}", cli.port))?;
    let port = listener.local_addr()?.port();

    let sim = PeerSimulator::new().await?;
    let (router, wallets) = start_bridge(&data_dir, &sim, &listener).await?;
    let ready = Ready { port, wallets };

    fs::write(&ready_path, serde_json::to_string_pretty(&ready)?)?;
    info!("Sandbox bridge listening on http://127.0.0.1:{port}");

    axum::serve(listener, router).await?;

    // The simulator has to outlive the server, since the wallets are connected to it.
    drop(sim);

    Ok(())
}

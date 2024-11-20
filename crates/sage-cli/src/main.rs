mod app_state;
mod router;
mod tls;

use anyhow::Result;
use app_state::AppState;
use axum_server::tls_rustls::RustlsConfig;
use router::api_router;
use rustls::crypto::aws_lc_rs::default_provider;
use sage::Sage;
use std::{net::SocketAddr, sync::Arc};
use tls::load_rustls_config;
use tokio::sync::Mutex;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    default_provider()
        .install_default()
        .expect("could not install AWS LC provider");

    let path = dirs::data_dir()
        .expect("could not find data directory")
        .join("com.rigidnetwork.sage");

    let mut app = Sage::new(&path);
    let mut receiver = app.initialize().await?;

    tokio::spawn(async move {
        while let Some(message) = receiver.recv().await {
            println!("{message:?}");
        }
    });

    let addr: SocketAddr = "0.0.0.0:3000".parse()?;
    info!("RPC server is listening at {addr}");

    let app = api_router().with_state(AppState {
        sage: Arc::new(Mutex::new(app)),
    });

    let config = load_rustls_config(
        path.join("ssl")
            .join("wallet.crt")
            .to_str()
            .expect("could not convert path to string"),
        path.join("ssl")
            .join("wallet.key")
            .to_str()
            .expect("could not convert path to string"),
    )?;

    axum_server::bind_rustls(addr, RustlsConfig::from_config(Arc::new(config)))
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

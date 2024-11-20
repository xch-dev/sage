use std::{net::SocketAddr, path::PathBuf, sync::Arc};

use anyhow::Result;
use axum_server::tls_rustls::RustlsConfig;
use sage::Sage;
use tokio::sync::Mutex;
use tracing::info;

use crate::{app_state::AppState, router::api_router, tls::load_rustls_config};

pub async fn start_rpc(path: PathBuf) -> Result<()> {
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

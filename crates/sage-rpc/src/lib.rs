mod tls;

use std::{net::SocketAddr, sync::Arc};

use anyhow::Result;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use axum_server::tls_rustls::RustlsConfig;
use sage::Sage;
use sage_api::ErrorKind;
use sage_api_macro::impl_endpoints;
use serde::Serialize;
use tokio::sync::Mutex;
use tracing::info;

#[derive(Debug, Clone)]
struct AppState {
    sage: Arc<Mutex<Sage>>,
}

impl_endpoints! {
    (repeat async fn endpoint(State(state): State<AppState>, Json(req): Json<sage_api::Endpoint>) -> Response {
        handle(state.sage.lock().await.endpoint(req) maybe_await)
    })

    fn api_router() -> Router<AppState> {
        Router::new()
            (repeat .route(&format!("/{}", endpoint_string), post(endpoint)))
    }
}

fn handle<T>(value: sage::Result<T>) -> Response
where
    T: Serialize,
{
    match value {
        Ok(data) => Json(data).into_response(),
        Err(error) => {
            let status = match error.kind() {
                ErrorKind::Api => StatusCode::BAD_REQUEST,
                ErrorKind::NotFound => StatusCode::NOT_FOUND,
                ErrorKind::Unauthorized => StatusCode::UNAUTHORIZED,
                ErrorKind::Wallet | ErrorKind::Internal => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, error.to_string()).into_response()
        }
    }
}

pub async fn start_rpc(sage: Arc<Mutex<Sage>>) -> Result<()> {
    let app = sage.lock().await;

    let addr: SocketAddr = ([127, 0, 0, 1], app.config.rpc.port).into();
    info!("RPC server is listening at {addr}");

    let config = tls::load_rustls_config(
        app.path
            .join("ssl")
            .join("wallet.crt")
            .to_str()
            .expect("could not convert path to string"),
        app.path
            .join("ssl")
            .join("wallet.key")
            .to_str()
            .expect("could not convert path to string"),
    )?;

    drop(app);

    let router = api_router().with_state(AppState { sage });

    axum_server::bind_rustls(addr, RustlsConfig::from_config(Arc::new(config)))
        .serve(router.into_make_service())
        .await?;

    Ok(())
}

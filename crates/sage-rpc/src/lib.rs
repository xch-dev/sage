mod cert_verifier;
mod openapi;
mod rustls_config;

#[cfg(test)]
mod tests;

use std::{net::SocketAddr, sync::Arc};

use anyhow::Result;
use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
};

use axum_server::tls_rustls::RustlsConfig;
use rustls_config::load_rustls_config;
use sage::Sage;
use sage_api::{ErrorKind, RecipientError, RecipientErrorKind};
use sage_api_macro::impl_endpoints;
// Aliased because `impl_endpoints!` rewrites every identifier containing `endpoint`.
use sage_auth::{Decision, Identity, authorize_endpoint as authorize_api};
use serde::Serialize;
use tokio::sync::Mutex;
use tracing::info;

// Re-export for CLI usage
pub use openapi::generate_openapi as generate_openapi_spec;

#[derive(Debug, Clone)]
struct AppState {
    sage: Arc<Mutex<Sage>>,
}

// `unused_mut` is allowed because some endpoints take `&self`.
impl_endpoints! {
    (repeat #[allow(unused_mut)] async fn endpoint(State(state): State<AppState>, Json(req): Json<sage_api::Endpoint>) -> Response {
        let mut sage = state.sage.lock().await;

        match authorize_api(&Identity::Owner, endpoint_string, sage.config.global.fingerprint) {
            Ok(Decision::Allowed) => {}
            Ok(_) => {
                return (StatusCode::FORBIDDEN, "this action needs approval, which isn't supported")
                    .into_response();
            }
            Err(error) => return (StatusCode::FORBIDDEN, error.to_string()).into_response(),
        }

        handle(sage.endpoint(req) maybe_await)
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
            if let Some(recipients) = error.recipient_errors() {
                return recipient_error_response(error.to_string(), recipients);
            }

            let status = match error.kind() {
                ErrorKind::Api | ErrorKind::Recipient => StatusCode::BAD_REQUEST,
                ErrorKind::NotFound => StatusCode::NOT_FOUND,
                ErrorKind::Unauthorized => StatusCode::UNAUTHORIZED,
                ErrorKind::AlreadyRunning => StatusCode::CONFLICT,
                ErrorKind::DatabaseMigration
                | ErrorKind::SharedDatabaseMigration
                | ErrorKind::Wallet
                | ErrorKind::Internal
                | ErrorKind::Nfc => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, error.to_string()).into_response()
        }
    }
}

/// The body of an error caused by recipients which couldn't be used.
#[derive(Serialize)]
struct RecipientErrorBody<'a> {
    kind: ErrorKind,
    reason: String,
    recipients: &'a [RecipientError],
}

/// Recipient errors are returned as JSON, so that callers can tell which recipients failed,
/// and confirm a changed handle by sending its new address in `confirmed_handles`. A handle
/// which changed is a conflict with what the caller expected, rather than a bad request.
fn recipient_error_response(reason: String, recipients: &[RecipientError]) -> Response {
    let status = if recipients
        .iter()
        .any(|recipient| recipient.kind == RecipientErrorKind::Changed)
    {
        StatusCode::CONFLICT
    } else {
        StatusCode::BAD_REQUEST
    };

    let body = RecipientErrorBody {
        kind: ErrorKind::Recipient,
        reason,
        recipients,
    };

    (status, Json(body)).into_response()
}

pub async fn start_rpc(sage: Arc<Mutex<Sage>>) -> Result<()> {
    let app = sage.lock().await;

    let addr: SocketAddr = ([127, 0, 0, 1], app.config.rpc.port).into();
    info!("RPC server is listening at {addr}");

    let config = load_rustls_config(
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

    let router = make_router(sage);

    axum_server::bind_rustls(addr, RustlsConfig::from_config(Arc::new(config)))
        .serve(router.into_make_service())
        .await?;

    Ok(())
}

pub fn make_router(sage: Arc<Mutex<Sage>>) -> Router {
    api_router().with_state(AppState { sage })
}

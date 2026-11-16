mod operation_approvals;
pub use operation_approvals::{OperationReview, RpcApprovalRequest};
mod cert_verifier;
mod credentials;
mod openapi;
mod rustls_config;
mod scoped_server;
mod session;
pub use credentials::*;
pub use scoped_server::*;

pub use session::RpcSession;

#[cfg(test)]
mod tests;

use std::{net::SocketAddr, sync::Arc};

use anyhow::Result;
use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
};

use axum_server::tls_rustls::RustlsConfig;
use rustls_config::load_rustls_config;
use sage::Sage;
use sage_api::{ErrorKind, RecipientError, RecipientErrorKind};
use sage_api_macro::impl_endpoints;
use serde::Serialize;
use tokio::sync::Mutex;
use tracing::info;

// Re-export for CLI usage
pub use openapi::generate_openapi as generate_openapi_spec;

#[derive(Debug, Clone)]
struct AppState {
    sage: Arc<Mutex<Sage>>,
    authenticator: Option<Arc<dyn RpcAuthenticator>>,
}

impl_endpoints! {
    (repeat async fn endpoint(State(state): State<AppState>, headers: HeaderMap, Json(req): Json<sage_api::Endpoint>) -> Response {
        let mut session = match state.session(&headers, endpoint_string, &req).await {
            Ok(session) => session,
            Err(error) => return handle::<()>(Err(error)),
        };
        handle(session.endpoint(req) maybe_await)
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
    api_router().with_state(AppState {
        sage,
        authenticator: None,
    })
}

/// Trusted authentication/grant resolver for a scoped HTTP listener. It must validate the
/// credentials and read current grants on each invocation. Client-provided identities or
/// capability lists are never accepted by the router itself. This callback runs under the
/// Sage lock, so it must not try to acquire that lock again.
pub trait RpcAuthenticator: std::fmt::Debug + Send + Sync {
    fn authenticate(&self, headers: &HeaderMap) -> Result<sage_permissions::Caller, String>;
    fn request_approval(
        &self,
        _request: RpcApprovalRequest,
    ) -> Result<
        tokio::sync::oneshot::Receiver<Result<sage_permissions::ApprovalBinding, String>>,
        String,
    > {
        Err("This transport has no approval presenter".into())
    }
}

/// Build a restricted RPC listener. Missing/invalid credentials fail closed; there is no
/// fallback to owner access. GUI-required operations are denied on this transport until a
/// host approval adapter is supplied. Existing mTLS owner RPC continues to use `make_router`.
pub fn make_scoped_router(
    sage: Arc<Mutex<Sage>>,
    authenticator: Arc<dyn RpcAuthenticator>,
) -> Router {
    api_router().with_state(AppState {
        sage,
        authenticator: Some(authenticator),
    })
}

impl AppState {
    async fn session(
        &self,
        headers: &HeaderMap,
        method: &str,
        request: &impl Serialize,
    ) -> sage::Result<RpcSession<'_>> {
        let sage = self.sage.lock().await;
        let Some(authenticator) = &self.authenticator else {
            return Ok(RpcSession::owner(sage));
        };
        let caller = authenticator
            .authenticate(headers)
            .map_err(sage::Error::PermissionDenied)?;
        let caller_id = caller.id.clone();
        let body = serde_json::to_value(request)
            .map_err(|e| sage::Error::PermissionDenied(e.to_string()))?;
        let mut session = RpcSession::scoped(sage, caller, String::new(), None);
        let (approval, fingerprint) = session.preflight_rpc(method, &body)?;
        if !approval {
            return Ok(session);
        }
        let (network_name, network_context) = session.network_context()?;
        // Never hold the wallet mutex while the user reviews a request.
        let pending = authenticator
            .request_approval(RpcApprovalRequest {
                caller: caller_id.clone(),
                method: method.into(),
                body,
                fingerprint,
                network: network_name,
            })
            .map_err(sage::Error::PermissionDenied)?;
        drop(session);
        let binding = tokio::time::timeout(std::time::Duration::from_secs(300), pending)
            .await
            .map_err(|_| sage::Error::PermissionDenied("Approval timed out".into()))?
            .map_err(|_| sage::Error::PermissionDenied("Approval canceled".into()))?
            .map_err(sage::Error::PermissionDenied)?;
        let sage = self.sage.lock().await;
        let caller = authenticator
            .authenticate(headers)
            .map_err(sage::Error::PermissionDenied)?;
        if caller.id != caller_id {
            return Err(sage::Error::PermissionDenied(
                "Caller changed during approval".into(),
            ));
        }
        // Authorization rechecks current grants and the bound wallet/payload during execution.
        if serde_json::to_string(&sage.network())
            .map_err(|e| sage::Error::PermissionDenied(e.to_string()))?
            != network_context
        {
            return Err(sage::Error::PermissionDenied(
                "Network changed during approval".into(),
            ));
        }
        Ok(RpcSession::scoped(
            sage,
            caller,
            String::new(),
            Some(binding),
        ))
    }
}

use crate::{ConnectionGrant, CredentialStore, credential_now};
use anyhow::{Context, Result};
use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Request, State},
    http::{StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::post,
};
use axum_server::{Handle, tls_rustls::RustlsConfig};
use serde::{Deserialize, Serialize};
use std::{
    net::{Ipv4Addr, SocketAddr},
    path::Path,
    sync::Arc,
};
use tokio::sync::Mutex;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PairingRequest {
    grant: ConnectionGrant,
    secret_hash: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CollectRequest {
    id: String,
    secret: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CollectResponse {
    status: &'static str,
    credential: Option<crate::IssuedCredential>,
}

async fn request_pairing(
    State(store): State<CredentialStore>,
    Json(request): Json<PairingRequest>,
) -> Response {
    match store.request_pairing(request.grant, request.secret_hash, credential_now()) {
        Ok(pending) => (StatusCode::ACCEPTED, Json(pending)).into_response(),
        Err(error) => (StatusCode::BAD_REQUEST, error.to_string()).into_response(),
    }
}
async fn collect_pairing(
    State(store): State<CredentialStore>,
    Json(request): Json<CollectRequest>,
) -> Response {
    match store.collect(&request.id, &request.secret, credential_now()) {
        Ok(credential) => Json(CollectResponse {
            status: if credential.is_some() {
                "approved"
            } else {
                "pending"
            },
            credential,
        })
        .into_response(),
        Err(error) => (StatusCode::BAD_REQUEST, error.to_string()).into_response(),
    }
}

async fn local_requests(State(port): State<u16>, request: Request, next: Next) -> Response {
    // Native local clients only. Browser origins are never accepted, including Origin: null.
    let expected = format!("127.0.0.1:{port}");
    let localhost = format!("localhost:{port}");
    let host = request
        .headers()
        .get(header::HOST)
        .and_then(|value| value.to_str().ok());
    if request.headers().contains_key(header::ORIGIN)
        || !matches!(host, Some(value) if value == expected || value == localhost)
    {
        return (
            StatusCode::FORBIDDEN,
            "Only local native clients are accepted",
        )
            .into_response();
    }
    let mut response = next.run(request).await;
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        header::HeaderValue::from_static("no-store"),
    );
    response
}

pub fn scoped_connection_router(
    sage: Arc<Mutex<sage::Sage>>,
    store: CredentialStore,
    port: u16,
) -> Router {
    let pairing = Router::new()
        .route("/pairing/request", post(request_pairing))
        .route("/pairing/collect", post(collect_pairing))
        .with_state(store.clone())
        .layer(DefaultBodyLimit::max(16 * 1024));
    crate::make_scoped_router(sage, Arc::new(store))
        .merge(pairing)
        .layer(middleware::from_fn_with_state(port, local_requests))
}

pub struct ScopedServer {
    pub port: u16,
    handle: Handle,
    task: tokio::task::JoinHandle<std::io::Result<()>>,
}
impl ScopedServer {
    pub fn is_running(&self) -> bool {
        !self.task.is_finished()
    }
    pub fn stop(self) {
        self.handle.shutdown();
        self.task.abort();
    }
}

/// A dedicated server certificate for the bearer listener. Never reuses the owner client key.
pub async fn start_scoped_server(
    sage: Arc<Mutex<sage::Sage>>,
    store: CredentialStore,
    root: &Path,
    port: u16,
) -> Result<ScopedServer> {
    let dir = root.join("scoped-rpc");
    std::fs::create_dir_all(&dir)?;
    let cert_path = dir.join("server.crt");
    let key_path = dir.join("server.key");
    if !cert_path.try_exists()? && !key_path.try_exists()? {
        let generated =
            rcgen::generate_simple_self_signed(vec!["localhost".into(), "127.0.0.1".into()])?;
        // New files are created exclusively, with owner-only key access on Unix.
        use std::io::Write;
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        options
            .open(&key_path)?
            .write_all(generated.key_pair.serialize_pem().as_bytes())?;
        options
            .open(&cert_path)?
            .write_all(generated.cert.pem().as_bytes())?;
    }
    let tls = RustlsConfig::from_pem_file(&cert_path, &key_path)
        .await
        .context("Could not load scoped RPC server certificate")?;
    let listener = std::net::TcpListener::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, port)))?;
    listener.set_nonblocking(true)?;
    let handle = Handle::new();
    let server = axum_server::from_tcp_rustls(listener, tls).handle(handle.clone());
    let router = scoped_connection_router(sage, store, port);
    let task = tokio::spawn(server.serve(router.into_make_service()));
    Ok(ScopedServer { port, handle, task })
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;
    #[tokio::test]
    async fn local_listener_rejects_browser_origins_and_never_falls_back_to_owner() {
        let root = tempfile::tempdir().unwrap();
        let store = CredentialStore::open(root.path()).unwrap();
        let router = scoped_connection_router(
            Arc::new(Mutex::new(sage::Sage::new(root.path(), true))),
            store,
            9258,
        );
        for (host, origin, expected) in [
            ("evil.example:9258", None, StatusCode::FORBIDDEN),
            ("localhost:9258", Some("null"), StatusCode::FORBIDDEN),
            ("localhost:9258", None, StatusCode::UNAUTHORIZED),
        ] {
            let mut request = Request::post("/get_version")
                .header("host", host)
                .header("content-type", "application/json");
            if let Some(origin) = origin {
                request = request.header("origin", origin);
            }
            let response = router
                .clone()
                .oneshot(request.body(Body::from("{}")).unwrap())
                .await
                .unwrap();
            assert_eq!(response.status(), expected);
        }
    }
}

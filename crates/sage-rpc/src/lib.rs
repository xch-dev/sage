mod openapi;

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
use rustls::{ServerConfig, pki_types::PrivateKeyDer};
use sage::Sage;
use sage_api::ErrorKind;
use sage_api_macro::impl_endpoints;
use serde::Serialize;
use tokio::sync::Mutex;
use tracing::info;

// Re-export for CLI usage
pub use openapi::generate_openapi as generate_openapi_spec;

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

/// Creates the API router without TLS. Used by integration tests.
pub fn make_router(sage: Arc<Mutex<Sage>>) -> Router {
    api_router().with_state(AppState { sage })
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
                ErrorKind::DatabaseMigration
                | ErrorKind::Wallet
                | ErrorKind::Internal
                | ErrorKind::Nfc => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, error.to_string()).into_response()
        }
    }
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

fn load_rustls_config(cert_path: &str, key_path: &str) -> Result<ServerConfig> {
    use anyhow::anyhow;
    use std::fs;

    let certs = {
        let cert_file = fs::read(cert_path)?;
        rustls_pemfile::certs(&mut cert_file.as_slice())
            .map(|item| item.map_err(|_| anyhow!("Failed to parse certificate")))
            .collect::<Result<Vec<_>>>()?
    };

    if certs.is_empty() {
        anyhow::bail!("No certificates found in {cert_path}");
    }

    let mut private_keys = {
        let key_file = fs::read(key_path)?;
        rustls_pemfile::pkcs8_private_keys(&mut key_file.as_slice())
            .map(|item| item.map_err(|_| anyhow!("Failed to parse key")))
            .collect::<Result<Vec<_>>>()?
    };

    if private_keys.is_empty() {
        anyhow::bail!("No private keys found in {key_path}");
    }

    let client_cert_verifier = Arc::new(WalletCertVerifier {
        wallet_cert: certs[0].as_ref().to_vec(),
    });

    let config = ServerConfig::builder()
        .with_client_cert_verifier(client_cert_verifier)
        .with_single_cert(certs, PrivateKeyDer::Pkcs8(private_keys.remove(0)))?;

    Ok(config)
}

#[derive(Debug)]
struct WalletCertVerifier {
    wallet_cert: Vec<u8>,
}

impl rustls::server::danger::ClientCertVerifier for WalletCertVerifier {
    fn root_hint_subjects(&self) -> &[rustls::DistinguishedName] {
        &[]
    }

    fn verify_client_cert(
        &self,
        end_entity: &rustls::pki_types::CertificateDer<'_>,
        _intermediates: &[rustls::pki_types::CertificateDer<'_>],
        _now: rustls::pki_types::UnixTime,
    ) -> Result<rustls::server::danger::ClientCertVerified, rustls::Error> {
        if end_entity.as_ref() == self.wallet_cert {
            Ok(rustls::server::danger::ClientCertVerified::assertion())
        } else {
            Err(rustls::Error::General(
                "Client certificate not allowed".into(),
            ))
        }
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &rustls::pki_types::CertificateDer<'_>,
        dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        use rustls::crypto::{aws_lc_rs::default_provider, verify_tls12_signature};
        verify_tls12_signature(
            message,
            cert,
            dss,
            &default_provider().signature_verification_algorithms,
        )
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &rustls::pki_types::CertificateDer<'_>,
        dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        use rustls::crypto::{aws_lc_rs::default_provider, verify_tls13_signature};
        verify_tls13_signature(
            message,
            cert,
            dss,
            &default_provider().signature_verification_algorithms,
        )
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        use rustls::crypto::aws_lc_rs::default_provider;
        default_provider()
            .signature_verification_algorithms
            .supported_schemes()
    }
}

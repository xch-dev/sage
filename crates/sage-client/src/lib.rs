mod error;

pub use error::*;

use std::{
    env,
    fs::{self, File},
    io::Read,
    net::SocketAddr,
    path::Path,
};

use reqwest::{Identity, StatusCode};
use sage_api_macro::impl_endpoints;
use sage_config::Config;
use serde::{de::DeserializeOwned, Serialize};

#[derive(Debug, Clone)]
pub struct Client {
    addr: SocketAddr,
    client: reqwest::Client,
}

impl Client {
    pub fn new() -> Result<Self, SageRpcError> {
        let path = dirs::data_dir()
            .ok_or(SageRpcError::MissingDataDir)?
            .join("com.rigidnetwork.sage");
        Self::from_dir(&path)
    }

    pub fn from_addr_and_identity(
        addr: SocketAddr,
        identity: Identity,
    ) -> Result<Self, SageRpcError> {
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .use_rustls_tls()
            .identity(identity)
            .build()?;

        Ok(Self { addr, client })
    }

    pub fn from_dir(path: &Path) -> Result<Self, SageRpcError> {
        let addr = if let Ok(addr) = env::var("SAGE_RPC_HOST") {
            addr.parse::<SocketAddr>()?
        } else {
            let config_path = path.join("config.toml");
            let config = if config_path.try_exists()? {
                let text = fs::read_to_string(&config_path)?;
                toml::from_str(&text)?
            } else {
                Config::default()
            };
            ([127, 0, 0, 1], config.rpc.port).into()
        };

        let cert_path = if let Ok(cert_path) = env::var("SAGE_RPC_CERT_PATH") {
            cert_path
        } else {
            path.join("ssl")
                .join("wallet.crt")
                .to_str()
                .ok_or(SageRpcError::InvalidPath)?
                .to_string()
        };

        let key_path = if let Ok(key_path) = env::var("SAGE_RPC_KEY_PATH") {
            key_path
        } else {
            path.join("ssl")
                .join("wallet.key")
                .to_str()
                .ok_or(SageRpcError::InvalidPath)?
                .to_string()
        };

        let mut buf = Vec::new();
        File::open(cert_path)?.read_to_end(&mut buf)?;
        File::open(key_path)?.read_to_end(&mut buf)?;
        let identity = Identity::from_pem(&buf)?;

        Self::from_addr_and_identity(addr, identity)
    }

    async fn call_rpc<T: Serialize, R: DeserializeOwned>(
        &self,
        url: &str,
        body: T,
    ) -> Result<R, SageRpcError> {
        let response = self
            .client
            .post(format!("https://{}{}", self.addr, url))
            .json(&body)
            .send()
            .await?;

        if response.status() != StatusCode::OK {
            return Err(SageRpcError::Api(response.status(), response.text().await?));
        }

        Ok(response.json::<R>().await?)
    }
}

impl_endpoints! {
    impl Client {
        (repeat pub async fn endpoint(&self, body: sage_api::Endpoint) -> Result<sage_api::EndpointResponse, SageRpcError> {
            self.call_rpc(&format!("/{}", endpoint_string), body).await
        })
    }
}

use std::net::SocketAddr;

use chia::protocol::Bytes32;
use hex_literal::hex;
use serde::{Deserialize, Serialize};
use serde_with::{hex::Hex, serde_as};
use tracing::{error, instrument};

use crate::ClientError;

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Network {
    pub default_port: u16,
    #[serde_as(as = "Hex")]
    pub genesis_challenge: Bytes32,
    #[serde_as(as = "Option<Hex>")]
    pub agg_sig_me: Option<Bytes32>,
    pub dns_introducers: Vec<String>,
}

impl Network {
    pub fn default_mainnet() -> Self {
        Self {
            default_port: 8444,
            genesis_challenge: Bytes32::new(hex!(
                "ccd5bb71183532bff220ba46c268991a3ff07eb358e8255a65c30a2dce0e5fbb"
            )),
            agg_sig_me: None,
            dns_introducers: vec![
                "dns-introducer.chia.net".to_string(),
                "chia.ctrlaltdel.ch".to_string(),
                "seeder.dexie.space".to_string(),
                "chia.hoffmang.com".to_string(),
            ],
        }
    }

    pub fn default_testnet11() -> Self {
        Self {
            default_port: 58444,
            genesis_challenge: Bytes32::new(hex!(
                "37a90eb5185a9c4439a91ddc98bbadce7b4feba060d50116a067de66bf236615"
            )),
            agg_sig_me: None,
            dns_introducers: vec!["dns-introducer-testnet11.chia.net".to_string()],
        }
    }

    #[instrument]
    pub async fn lookup_all(&self) -> Vec<SocketAddr> {
        let mut result = Vec::new();
        for dns_introducer in &self.dns_introducers {
            match self.lookup_host(dns_introducer).await {
                Ok(addrs) => result.extend(addrs),
                Err(error) => error!("Failed to lookup DNS introducer {dns_introducer}: {error}"),
            }
        }
        result
    }

    #[instrument]
    pub async fn lookup_host(&self, dns_introducer: &str) -> Result<Vec<SocketAddr>, ClientError> {
        let addrs = tokio::net::lookup_host(format!("{dns_introducer}:80")).await?;
        let mut result = Vec::new();
        for addr in addrs {
            result.push(SocketAddr::new(addr.ip(), self.default_port));
        }
        Ok(result)
    }
}

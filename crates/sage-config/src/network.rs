use chia::protocol::Bytes32;
use chia_wallet_sdk::types::{MAINNET_CONSTANTS, TESTNET11_CONSTANTS};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_with::{hex::Hex, serde_as};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct NetworkList {
    pub networks: Vec<Network>,
}

impl Default for NetworkList {
    fn default() -> Self {
        Self {
            // These will inherit from the mainnet and testnet11 networks anyway
            // So we don't need to include the introducers in the config
            // The idea is if we add new introducers over time, they will automatically
            // be added to the mainnet and testnet11 networks for everyone
            networks: vec![
                Network {
                    additional_dns_introducers: Vec::new(),
                    additional_peer_introducers: Vec::new(),
                    ..MAINNET.clone()
                },
                Network {
                    additional_dns_introducers: Vec::new(),
                    additional_peer_introducers: Vec::new(),
                    ..TESTNET11.clone()
                },
            ],
        }
    }
}

impl NetworkList {
    pub fn by_name(&self, name: &str) -> Option<&Network> {
        self.networks.iter().find(|network| network.name == name)
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Network {
    pub name: String,
    pub ticker: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,
    #[serde(
        default = "default_precision",
        skip_serializing_if = "is_default_precision"
    )]
    pub precision: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub network_id: Option<String>,
    pub default_port: u16,
    #[serde_as(as = "Hex")]
    #[specta(type = String)]
    pub genesis_challenge: Bytes32,
    #[serde_as(as = "Option<Hex>")]
    #[specta(type = Option<String>)]
    pub agg_sig_me: Option<Bytes32>,
    #[serde(
        default,
        skip_serializing_if = "Vec::is_empty",
        rename = "dns_introducers"
    )]
    pub additional_dns_introducers: Vec<String>,
    #[serde(
        default,
        skip_serializing_if = "Vec::is_empty",
        rename = "peer_introducers"
    )]
    pub additional_peer_introducers: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inherit: Option<InheritedNetwork>,
}

impl Network {
    pub fn prefix(&self) -> String {
        self.prefix
            .clone()
            .unwrap_or_else(|| self.ticker.to_lowercase())
    }

    pub fn network_id(&self) -> String {
        self.network_id.clone().unwrap_or_else(|| self.name.clone())
    }

    pub fn agg_sig_me(&self) -> Bytes32 {
        self.agg_sig_me.unwrap_or(self.genesis_challenge)
    }

    pub fn dns_introducers(&self) -> Vec<String> {
        match self.inherit {
            Some(InheritedNetwork::Mainnet) => {
                let mut introducers = self.additional_dns_introducers.clone();
                for introducer in &MAINNET.additional_dns_introducers {
                    if !introducers.contains(introducer) {
                        introducers.push(introducer.clone());
                    }
                }
                introducers
            }
            Some(InheritedNetwork::Testnet11) => {
                let mut introducers = self.additional_dns_introducers.clone();
                for introducer in &TESTNET11.additional_dns_introducers {
                    if !introducers.contains(introducer) {
                        introducers.push(introducer.clone());
                    }
                }
                introducers
            }
            None => self.additional_dns_introducers.clone(),
        }
    }

    pub fn peer_introducers(&self) -> Vec<String> {
        match self.inherit {
            Some(InheritedNetwork::Mainnet) => {
                let mut introducers = self.additional_peer_introducers.clone();
                for introducer in &MAINNET.additional_peer_introducers {
                    if !introducers.contains(introducer) {
                        introducers.push(introducer.clone());
                    }
                }
                introducers
            }
            Some(InheritedNetwork::Testnet11) => {
                let mut introducers = self.additional_peer_introducers.clone();
                for introducer in &TESTNET11.additional_peer_introducers {
                    if !introducers.contains(introducer) {
                        introducers.push(introducer.clone());
                    }
                }
                introducers
            }
            None => self.additional_peer_introducers.clone(),
        }
    }
}

fn default_precision() -> u8 {
    12
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_default_precision(precision: &u8) -> bool {
    *precision == 12
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
pub enum InheritedNetwork {
    #[serde(rename = "mainnet")]
    Mainnet,
    #[serde(rename = "testnet11")]
    Testnet11,
}

pub static MAINNET: Lazy<Network> = Lazy::new(|| Network {
    name: "mainnet".to_string(),
    network_id: None,
    default_port: 8444,
    ticker: "XCH".to_string(),
    prefix: None,
    precision: 12,
    genesis_challenge: MAINNET_CONSTANTS.genesis_challenge,
    agg_sig_me: None,
    additional_dns_introducers: vec![
        "dns-introducer.chia.net".to_string(),
        "chia.ctrlaltdel.ch".to_string(),
        "seeder.dexie.space".to_string(),
        "chia.hoffmang.com".to_string(),
    ],
    additional_peer_introducers: vec!["introducer.chia.net".to_string()],
    inherit: Some(InheritedNetwork::Mainnet),
});

pub static TESTNET11: Lazy<Network> = Lazy::new(|| Network {
    name: "testnet11".to_string(),
    network_id: None,
    default_port: 58444,
    ticker: "TXCH".to_string(),
    prefix: None,
    precision: 12,
    genesis_challenge: TESTNET11_CONSTANTS.genesis_challenge,
    agg_sig_me: None,
    additional_dns_introducers: vec!["dns-introducer-testnet11.chia.net".to_string()],
    additional_peer_introducers: vec!["introducer-testnet11.chia.net".to_string()],
    inherit: Some(InheritedNetwork::Testnet11),
});

use std::num::ParseIntError;

use chia::protocol::Bytes32;
use indexmap::IndexMap;
use serde::Deserialize;
use serde_with::{hex::Hex, serde_as};
use specta::Type;

use crate::{
    ChangeMode, Config, DerivationMode, GlobalConfig, InheritedNetwork, Network, NetworkConfig,
    NetworkList, RpcConfig, Wallet, WalletConfig, WalletDefaults,
};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Type)]
#[serde(default)]
pub struct OldConfig {
    version: u32,
    app: OldAppConfig,
    rpc: OldRpcConfig,
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    wallets: IndexMap<String, OldWalletConfig>,
    network: OldNetworkConfig,
}

impl OldConfig {
    pub fn is_old(&self) -> bool {
        self.version == 1
    }
}

impl Default for OldConfig {
    fn default() -> Self {
        Self {
            version: 1,
            app: OldAppConfig::default(),
            rpc: OldRpcConfig::default(),
            wallets: IndexMap::new(),
            network: OldNetworkConfig::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Type)]
#[serde(default)]
struct OldWalletConfig {
    name: String,
    derive_automatically: bool,
    derivation_batch_size: u32,
}

impl Default for OldWalletConfig {
    fn default() -> Self {
        Self {
            name: "Unnamed Wallet".to_string(),
            derive_automatically: true,
            derivation_batch_size: 500,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Type)]
#[serde(default)]
struct OldRpcConfig {
    run_on_startup: bool,
    server_port: u16,
}

impl Default for OldRpcConfig {
    fn default() -> Self {
        Self {
            run_on_startup: false,
            server_port: 9257,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Type)]
#[serde(default)]
struct OldAppConfig {
    log_level: String,
    active_fingerprint: Option<u32>,
}

impl Default for OldAppConfig {
    fn default() -> Self {
        Self {
            log_level: "INFO".to_string(),
            active_fingerprint: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Type)]
#[serde(default)]
struct OldNetworkConfig {
    network_id: String,
    target_peers: u32,
    discover_peers: bool,
}

impl Default for OldNetworkConfig {
    fn default() -> Self {
        Self {
            network_id: "mainnet".to_string(),
            target_peers: 5,
            discover_peers: true,
        }
    }
}

#[serde_as]
#[derive(Debug, Clone, Deserialize, Type)]
pub struct OldNetwork {
    default_port: u16,
    ticker: String,
    address_prefix: String,
    precision: u8,
    #[serde_as(as = "Hex")]
    #[specta(type = String)]
    genesis_challenge: Bytes32,
    #[serde_as(as = "Hex")]
    #[specta(type = String)]
    agg_sig_me: Bytes32,
    dns_introducers: Vec<String>,
}

pub fn migrate_config(old: OldConfig) -> Result<(Config, WalletConfig), ParseIntError> {
    let config = Config {
        version: 2,
        global: GlobalConfig {
            log_level: old.app.log_level,
            fingerprint: old.app.active_fingerprint,
        },
        network: NetworkConfig {
            default_network: old.network.network_id,
            target_peers: old.network.target_peers,
            discover_peers: old.network.discover_peers,
        },
        rpc: RpcConfig {
            enabled: old.rpc.run_on_startup,
            port: old.rpc.server_port,
        },
    };

    let mut wallet_config = WalletConfig {
        defaults: WalletDefaults::default(),
        wallets: Vec::new(),
    };

    for (fingerprint, wallet) in old.wallets {
        wallet_config.wallets.push(Wallet {
            name: wallet.name,
            fingerprint: fingerprint.parse()?,
            derivation: if wallet.derive_automatically {
                DerivationMode::Default
            } else {
                DerivationMode::Static
            },
            change: ChangeMode::Default,
            network: None,
        });
    }

    Ok((config, wallet_config))
}

pub fn migrate_networks(old: IndexMap<String, OldNetwork>) -> NetworkList {
    NetworkList {
        networks: old
            .into_iter()
            .map(|(name, network)| {
                let expected_prefix = network.ticker.to_lowercase();
                let inherit = match name.as_str() {
                    "mainnet" => Some(InheritedNetwork::Mainnet),
                    "testnet11" => Some(InheritedNetwork::Testnet11),
                    _ => None,
                };

                Network {
                    name,
                    ticker: network.ticker,
                    prefix: if network.address_prefix == expected_prefix {
                        None
                    } else {
                        Some(network.address_prefix)
                    },
                    precision: network.precision,
                    network_id: None,
                    default_port: network.default_port,
                    genesis_challenge: network.genesis_challenge,
                    agg_sig_me: if network.agg_sig_me == network.genesis_challenge {
                        None
                    } else {
                        Some(network.agg_sig_me)
                    },
                    additional_dns_introducers: network.dns_introducers,
                    additional_peer_introducers: vec![],
                    inherit,
                }
            })
            .collect(),
    }
}

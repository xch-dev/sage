use std::num::ParseIntError;

use chia_wallet_sdk::prelude::*;
use indexmap::IndexMap;
use serde::Deserialize;
use serde_with::{hex::Hex, serde_as};
use specta::Type;

use crate::{
    Config, GlobalConfig, InheritedNetwork, Network, NetworkConfig, NetworkList, RpcConfig, Wallet,
    WalletConfig, WalletDefaults,
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
            network: None,
            delta_sync: None,
            emoji: None,
            change_address: None,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn old_config_default_is_v1() {
        let old = OldConfig::default();
        assert!(old.is_old());
    }

    #[test]
    fn migrate_config_basic() {
        let old = OldConfig::default();
        let (config, wallet_config) = migrate_config(old).unwrap();

        assert_eq!(config.version, 2);
        assert_eq!(config.global.log_level, "INFO");
        assert!(config.global.fingerprint.is_none());
        assert_eq!(config.network.default_network, "mainnet");
        assert_eq!(config.network.target_peers, 5);
        assert!(config.network.discover_peers);
        assert!(!config.rpc.enabled);
        assert_eq!(config.rpc.port, 9257);
        assert!(wallet_config.wallets.is_empty());
    }

    #[test]
    fn migrate_config_with_fingerprint_and_wallets() {
        let mut old = OldConfig::default();
        old.app.active_fingerprint = Some(12345);
        old.app.log_level = "DEBUG".to_string();
        old.rpc.run_on_startup = true;
        old.rpc.server_port = 8080;
        old.network.network_id = "testnet11".to_string();
        old.wallets.insert(
            "67890".to_string(),
            OldWalletConfig {
                name: "My Wallet".to_string(),
                ..OldWalletConfig::default()
            },
        );

        let (config, wallet_config) = migrate_config(old).unwrap();

        assert_eq!(config.global.fingerprint, Some(12345));
        assert_eq!(config.global.log_level, "DEBUG");
        assert!(config.rpc.enabled);
        assert_eq!(config.rpc.port, 8080);
        assert_eq!(config.network.default_network, "testnet11");
        assert_eq!(wallet_config.wallets.len(), 1);
        assert_eq!(wallet_config.wallets[0].fingerprint, 67890);
        assert_eq!(wallet_config.wallets[0].name, "My Wallet");
    }

    #[test]
    fn migrate_config_invalid_fingerprint_key() {
        let mut old = OldConfig::default();
        old.wallets.insert(
            "not_a_number".to_string(),
            OldWalletConfig::default(),
        );
        let result = migrate_config(old);
        assert!(result.is_err());
    }

    #[test]
    fn migrate_networks_mainnet_inherits() {
        let genesis = Bytes32::new([1; 32]);
        let mut networks = IndexMap::new();
        networks.insert(
            "mainnet".to_string(),
            OldNetwork {
                default_port: 8444,
                ticker: "XCH".to_string(),
                address_prefix: "xch".to_string(),
                precision: 12,
                genesis_challenge: genesis,
                agg_sig_me: genesis, // same as genesis → should become None
                dns_introducers: vec!["dns.example.com".to_string()],
            },
        );

        let result = migrate_networks(networks);
        assert_eq!(result.networks.len(), 1);
        let net = &result.networks[0];
        assert_eq!(net.name, "mainnet");
        assert!(net.prefix.is_none()); // matches lowercase ticker
        assert!(net.agg_sig_me.is_none()); // matches genesis
        assert!(matches!(net.inherit, Some(InheritedNetwork::Mainnet)));
    }

    #[test]
    fn migrate_networks_custom_prefix_preserved() {
        let genesis = Bytes32::new([2; 32]);
        let agg = Bytes32::new([3; 32]);
        let mut networks = IndexMap::new();
        networks.insert(
            "custom".to_string(),
            OldNetwork {
                default_port: 9999,
                ticker: "CUST".to_string(),
                address_prefix: "mycustom".to_string(), // doesn't match "cust"
                precision: 6,
                genesis_challenge: genesis,
                agg_sig_me: agg, // different from genesis
                dns_introducers: vec![],
            },
        );

        let result = migrate_networks(networks);
        let net = &result.networks[0];
        assert_eq!(net.prefix, Some("mycustom".to_string()));
        assert_eq!(net.agg_sig_me, Some(agg));
        assert!(net.inherit.is_none()); // not mainnet or testnet11
    }
}

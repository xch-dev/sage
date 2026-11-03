use std::sync::LazyLock;

use chia_wallet_sdk::prelude::*;
use hex_literal::hex;
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
                    xchandles: None,
                    ..MAINNET.clone()
                },
                Network {
                    additional_dns_introducers: Vec::new(),
                    additional_peer_introducers: Vec::new(),
                    xchandles: None,
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
    /// Overrides the XCHandles deployment for this network. When unset, the deployment
    /// is inherited from the built-in network named by `inherit`, if there is one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub xchandles: Option<XchandlesConfig>,
}

/// The XCHandles deployment used to resolve handles on a network.
///
/// The constants must match the ones committed to in the registry's launcher solution,
/// since they determine the registry's puzzle and therefore which slots are authentic.
#[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct XchandlesConfig {
    #[serde_as(as = "Hex")]
    #[specta(type = String)]
    pub registry_launcher_id: Bytes32,
    #[serde_as(as = "Hex")]
    #[specta(type = String)]
    pub price_singleton_launcher_id: Bytes32,
    #[serde_as(as = "Hex")]
    #[specta(type = String)]
    pub precommit_payout_puzzle_hash: Bytes32,
    pub relative_block_height: u32,
    pub api_url: String,
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

    /// The XCHandles deployment for this network, or `None` if XCHandles isn't supported.
    ///
    /// A network only inherits a built-in deployment if it shares that network's genesis
    /// challenge, so a custom network never silently resolves handles against mainnet.
    pub fn xchandles(&self) -> Option<XchandlesConfig> {
        if let Some(xchandles) = &self.xchandles {
            return Some(xchandles.clone());
        }

        let inherited = match self.inherit? {
            InheritedNetwork::Mainnet => &*MAINNET,
            InheritedNetwork::Testnet11 => &*TESTNET11,
        };

        if inherited.genesis_challenge != self.genesis_challenge {
            return None;
        }

        inherited.xchandles.clone()
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

pub static MAINNET: LazyLock<Network> = LazyLock::new(|| Network {
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
    // These constants are committed to in the registry's launcher solution.
    xchandles: Some(XchandlesConfig {
        registry_launcher_id: hex!(
            "6f314f69a9cf776311e1c7781a3013e4daae3696c71d8a04d049a5af1530f050"
        )
        .into(),
        price_singleton_launcher_id: hex!(
            "8b99540c8f96524ca785fa7b74116c9d52909641e7645f987ff680d3a42b4ca2"
        )
        .into(),
        precommit_payout_puzzle_hash: hex!(
            "00d8d5b037d779c1aabbec764e39fc7710c2ac9351483abe0465e2237e4a8e08"
        )
        .into(),
        relative_block_height: 32,
        api_url: "https://api.xchandles.com".to_string(),
    }),
});

pub static TESTNET11: LazyLock<Network> = LazyLock::new(|| Network {
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
    // These constants are committed to in the registry's launcher solution.
    xchandles: Some(XchandlesConfig {
        registry_launcher_id: hex!(
            "c98d2ea5fbaa6507c2d56598b81833c6c643f1ead0062c4641ac1f9d94331c4a"
        )
        .into(),
        price_singleton_launcher_id: hex!(
            "ba5ec3acca2c6551ad40a472c3a9a6fe3d998b6272fa558d91fbb35a737854ef"
        )
        .into(),
        precommit_payout_puzzle_hash: hex!(
            "764e9d674d2fa441f0f6f8fc5e749a17dde345ebe4a33536afd3ef417a3f8c90"
        )
        .into(),
        relative_block_height: 8,
        api_url: "https://testnet-api.xchandles.com".to_string(),
    }),
});

#[cfg(test)]
mod tests {
    use chia_wallet_sdk::{
        driver::{SpendContext, XchandlesConstants, XchandlesRegistry},
        puzzles::SINGLETON_LAUNCHER_HASH,
    };

    use super::*;

    /// The registry launcher solutions, as spent on chain. They commit to the deployment
    /// constants, so the built-in configuration must match them exactly.
    const MAINNET_LAUNCHER_PARENT: [u8; 32] =
        hex!("222d60927d72d1a432c9cf886ce5a03ae2fce51c660d2fba5ac49bab0c66fb1b");
    const MAINNET_LAUNCHER_SOLUTION: &str = "ffa0377ae246d10d2a748dbe4d7777b577bfc81d9159d4b5ffacaa5fbb57d96f0e51ff01ffffa0f6dd3090e988df98d5a06fde22837e8c381f714fbad56d215ad5a1fcb972ef96ff01ff8401e187e0ffffa09fe69982864e2b380a54ded33bd50e2065b0dfaba1bb946836bcd378b533587bffa012c9d7f473b815ffdf3f616f637a8ebdf9b3ba93559611d25c1486a892408ca3a081ff48981ca40bc8828c6769a07aaad2875d2bf382e083fdbb7a579b74f35b89ffffa06f314f69a9cf776311e1c7781a3013e4daae3696c71d8a04d049a5af1530f050ffa000d8d5b037d779c1aabbec764e39fc7710c2ac9351483abe0465e2237e4a8e08ff20ffa08b99540c8f96524ca785fa7b74116c9d52909641e7645f987ff680d3a42b4ca2808080";
    const TESTNET11_LAUNCHER_PARENT: [u8; 32] =
        hex!("dec882b914508fbecc92fbd6c063aacc0cf1b373a0ef665a9f7c3d558974025e");
    const TESTNET11_LAUNCHER_SOLUTION: &str = "ffa0877d56490957030b2a9dd7459dfc2f8eb6ad869fa4dfedaea8f069572bbe6e74ff01ffffa068f624b66e29bb1375e75b4d6cb39ce8b3e9fe19ccc4c77b8ad496b21a43f95aff01ff8401e187e0ffffa03cf9d6dd82f7584de6d852d9306eb8743148f882b4c74980f88f442419df474affa012c9d7f473b815ffdf3f616f637a8ebdf9b3ba93559611d25c1486a892408ca3a081ff48981ca40bc8828c6769a07aaad2875d2bf382e083fdbb7a579b74f35b89ffffa0c98d2ea5fbaa6507c2d56598b81833c6c643f1ead0062c4641ac1f9d94331c4affa0764e9d674d2fa441f0f6f8fc5e749a17dde345ebe4a33536afd3ef417a3f8c90ff08ffa0ba5ec3acca2c6551ad40a472c3a9a6fe3d998b6272fa558d91fbb35a737854ef808080";

    fn launcher_constants(parent: [u8; 32], solution: &str) -> XchandlesConstants {
        let mut ctx = SpendContext::new();
        let launcher_coin = Coin::new(parent.into(), SINGLETON_LAUNCHER_HASH.into(), 1);
        let solution = Program::from(hex::decode(solution).unwrap());
        let solution = ctx.alloc(&solution).unwrap();

        let (registry, ..) =
            XchandlesRegistry::from_launcher_solution(&mut ctx, launcher_coin, solution)
                .unwrap()
                .expect("not an XCHandles registry launcher");

        registry.info.constants
    }

    fn assert_matches_launcher(network: &Network, parent: [u8; 32], solution: &str) {
        let config = network.xchandles().expect("missing XCHandles config");
        let constants = launcher_constants(parent, solution);

        assert_eq!(constants.launcher_id, config.registry_launcher_id);
        assert_eq!(
            constants.price_singleton_launcher_id,
            config.price_singleton_launcher_id
        );
        assert_eq!(
            constants.precommit_payout_puzzle_hash,
            config.precommit_payout_puzzle_hash
        );
        assert_eq!(
            constants.relative_block_height,
            config.relative_block_height
        );
    }

    #[test]
    fn test_mainnet_xchandles_constants_match_launcher() {
        assert_matches_launcher(&MAINNET, MAINNET_LAUNCHER_PARENT, MAINNET_LAUNCHER_SOLUTION);
    }

    #[test]
    fn test_testnet11_xchandles_constants_match_launcher() {
        assert_matches_launcher(
            &TESTNET11,
            TESTNET11_LAUNCHER_PARENT,
            TESTNET11_LAUNCHER_SOLUTION,
        );
    }

    #[test]
    fn test_default_networks_inherit_xchandles() {
        let list = NetworkList::default();

        for (name, expected) in [("mainnet", &*MAINNET), ("testnet11", &*TESTNET11)] {
            let network = list.by_name(name).unwrap();
            assert!(network.xchandles.is_none());
            assert_eq!(network.xchandles(), expected.xchandles);
        }
    }

    #[test]
    fn test_custom_network_has_no_xchandles() {
        let network = Network {
            name: "simulator0".to_string(),
            genesis_challenge: Bytes32::new([7; 32]),
            inherit: None,
            xchandles: None,
            ..TESTNET11.clone()
        };

        assert_eq!(network.xchandles(), None);
    }

    #[test]
    fn test_inherit_requires_matching_genesis_challenge() {
        let network = Network {
            name: "fork".to_string(),
            genesis_challenge: Bytes32::new([7; 32]),
            inherit: Some(InheritedNetwork::Mainnet),
            xchandles: None,
            ..MAINNET.clone()
        };

        assert_eq!(network.xchandles(), None);
    }

    #[test]
    fn test_xchandles_override() {
        let custom = XchandlesConfig {
            registry_launcher_id: Bytes32::new([1; 32]),
            price_singleton_launcher_id: Bytes32::new([2; 32]),
            precommit_payout_puzzle_hash: Bytes32::new([3; 32]),
            relative_block_height: 4,
            api_url: "http://localhost:8080".to_string(),
        };

        let network = Network {
            name: "simulator0".to_string(),
            inherit: None,
            xchandles: Some(custom.clone()),
            ..TESTNET11.clone()
        };

        let toml = toml::to_string_pretty(&network).unwrap();
        let parsed: Network = toml::from_str(&toml).unwrap();
        assert_eq!(parsed.xchandles(), Some(custom));
    }
}

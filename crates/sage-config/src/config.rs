use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct Config {
    pub version: u32,
    pub global: GlobalConfig,
    pub network: NetworkConfig,
    pub rpc: RpcConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: 2,
            global: GlobalConfig::default(),
            network: NetworkConfig::default(),
            rpc: RpcConfig::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct GlobalConfig {
    pub log_level: String,
    pub fingerprint: Option<u32>,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            log_level: "INFO".to_string(),
            fingerprint: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct NetworkConfig {
    pub default_network: String,
    pub target_peers: u32,
    pub discover_peers: bool,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            default_network: "mainnet".to_string(),
            target_peers: 5,
            discover_peers: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Copy, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct RpcConfig {
    pub enabled: bool,
    pub port: u16,
}

impl Default for RpcConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            port: 9257,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_defaults() {
        let config = Config::default();
        assert_eq!(config.version, 2);
        assert_eq!(config.global.log_level, "INFO");
        assert!(config.global.fingerprint.is_none());
        assert_eq!(config.network.default_network, "mainnet");
        assert_eq!(config.network.target_peers, 5);
        assert!(config.network.discover_peers);
        assert!(!config.rpc.enabled);
        assert_eq!(config.rpc.port, 9257);
    }

    #[test]
    fn config_toml_round_trip() {
        let config = Config::default();
        let toml_str = toml::to_string(&config).unwrap();
        let parsed: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(config, parsed);
    }

    #[test]
    fn config_partial_deserialize() {
        // Only specify a few fields, rest should use defaults
        let toml_str = r#"
version = 2

[global]
log_level = "DEBUG"
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.global.log_level, "DEBUG");
        assert_eq!(config.network.default_network, "mainnet"); // default
        assert!(!config.rpc.enabled); // default
    }

    #[test]
    fn config_with_fingerprint() {
        let toml_str = r#"
version = 2

[global]
log_level = "INFO"
fingerprint = 12345

[network]
default_network = "testnet11"
target_peers = 3
discover_peers = false

[rpc]
enabled = true
port = 8080
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.global.fingerprint, Some(12345));
        assert_eq!(config.network.default_network, "testnet11");
        assert_eq!(config.network.target_peers, 3);
        assert!(!config.network.discover_peers);
        assert!(config.rpc.enabled);
        assert_eq!(config.rpc.port, 8080);
    }
}

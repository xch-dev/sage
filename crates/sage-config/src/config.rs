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
    /// The name of the UI theme the user last selected.
    pub theme: Option<String>,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            log_level: "INFO".to_string(),
            fingerprint: None,
            theme: None,
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
    fn test_config_without_theme() {
        let config: Config = toml::from_str(
            r#"
            version = 2

            [global]
            log_level = "DEBUG"
            fingerprint = 1000000
            "#,
        )
        .expect("Failed to parse config");

        assert_eq!(config.global.log_level, "DEBUG");
        assert_eq!(config.global.fingerprint, Some(1_000_000));
        assert_eq!(config.global.theme, None);
    }

    #[test]
    fn test_config_theme_round_trip() {
        let mut config = Config::default();
        config.global.theme = Some("dark".to_string());

        let toml = toml::to_string_pretty(&config).expect("Failed to serialize config");
        let parsed: Config = toml::from_str(&toml).expect("Failed to parse config");

        assert_eq!(parsed, config);
        assert_eq!(parsed.global.theme.as_deref(), Some("dark"));
    }
}

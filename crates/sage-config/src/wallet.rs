use chia_wallet_sdk::prelude::*;
use serde::{Deserialize, Serialize};
use serde_with::{hex::Hex, serde_as};
use specta::Type;

/// The current version of `wallets.toml`. Files written before it had a version are version 0.
///
/// - Version 1: the wallet list is authoritative. Before this, every BLS key in the keychain
///   was a wallet, so a key without an entry is recovered as a wallet once when upgrading.
///   From now on the keychain can also hold keys that aren't wallets.
pub const WALLET_CONFIG_VERSION: u32 = 1;

#[derive(Debug, Default, Clone, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct WalletConfig {
    pub version: u32,
    pub defaults: WalletDefaults,
    pub wallets: Vec<Wallet>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct WalletDefaults {
    pub delta_sync: bool,
}

impl Default for WalletDefaults {
    fn default() -> Self {
        Self { delta_sync: true }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct Wallet {
    pub name: String,
    pub fingerprint: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<String>,
    pub delta_sync: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emoji: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub change_address: Option<String>,
    /// What the wallet is made of. This is omitted for BLS wallets, which every wallet was
    /// before other kinds existed.
    // `default` comes last so that Specta marks the field optional.
    #[serde(skip_serializing_if = "WalletKind::is_bls", default)]
    pub kind: WalletKind,
}

/// The kind of a wallet. A BLS wallet's key is stored in the keychain under the wallet's
/// fingerprint. Other kinds are defined entirely here, since they have no key of their own.
#[serde_as]
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(tag = "type", rename_all = "snake_case")]
// Renamed so that it doesn't clash with the API's `WalletKind` in the TypeScript bindings.
#[specta(rename = "WalletConfigKind")]
pub enum WalletKind {
    #[default]
    Bls,
    Vault {
        #[serde_as(as = "Hex")]
        #[specta(type = String)]
        launcher_id: Bytes32,
    },
    Watch {
        #[serde_as(as = "Vec<Hex>")]
        #[specta(type = Vec<String>)]
        p2_puzzle_hashes: Vec<Bytes32>,
    },
}

impl WalletKind {
    pub fn is_bls(&self) -> bool {
        matches!(self, Self::Bls)
    }
}

impl Wallet {
    pub fn delta_sync(&self, defaults: &WalletDefaults) -> bool {
        self.delta_sync.unwrap_or(defaults.delta_sync)
    }
}

impl Default for Wallet {
    fn default() -> Self {
        Self {
            name: "Unnamed Wallet".to_string(),
            fingerprint: 0,
            network: None,
            delta_sync: None,
            emoji: None,
            change_address: None,
            kind: WalletKind::Bls,
        }
    }
}

#[cfg(test)]
mod tests {
    use expect_test::{Expect, expect};

    use super::*;

    fn default() -> Wallet {
        Wallet {
            fingerprint: 1_000_000,
            name: "Main".to_string(),
            network: None,
            delta_sync: None,
            emoji: None,
            change_address: Some(
                "xch1dtfukqqka3ftqtdlhmc5spc5vd44h7ejrtnjcewxlueam5yrnnqqyczg8t".to_string(),
            ),
            kind: WalletKind::Bls,
        }
    }

    fn check(value: Wallet, expect_toml: &Expect, expect_json: &Expect) {
        let value = WalletConfig {
            version: WALLET_CONFIG_VERSION,
            defaults: WalletDefaults::default(),
            wallets: vec![value],
        };
        let toml = toml::to_string_pretty(&value).expect("Failed to serialize toml");
        expect_toml.assert_eq(&toml);
        let json = serde_json::to_string_pretty(&value).expect("Failed to serialize json");
        expect_json.assert_eq(&json);
    }

    #[test]
    fn test_wallet_config_default() {
        let config = default();
        check(
            config,
            &expect![[r#"
                version = 1

                [defaults]
                delta_sync = true

                [[wallets]]
                name = "Main"
                fingerprint = 1000000
                change_address = "xch1dtfukqqka3ftqtdlhmc5spc5vd44h7ejrtnjcewxlueam5yrnnqqyczg8t"
            "#]],
            &expect![[r#"
                {
                  "version": 1,
                  "defaults": {
                    "delta_sync": true
                  },
                  "wallets": [
                    {
                      "name": "Main",
                      "fingerprint": 1000000,
                      "delta_sync": null,
                      "change_address": "xch1dtfukqqka3ftqtdlhmc5spc5vd44h7ejrtnjcewxlueam5yrnnqqyczg8t"
                    }
                  ]
                }"#]],
        );
    }

    #[test]
    fn test_wallet_config_override() {
        let config = Wallet { ..default() };
        check(
            config,
            &expect![[r#"
                version = 1

                [defaults]
                delta_sync = true

                [[wallets]]
                name = "Main"
                fingerprint = 1000000
                change_address = "xch1dtfukqqka3ftqtdlhmc5spc5vd44h7ejrtnjcewxlueam5yrnnqqyczg8t"
            "#]],
            &expect![[r#"
                {
                  "version": 1,
                  "defaults": {
                    "delta_sync": true
                  },
                  "wallets": [
                    {
                      "name": "Main",
                      "fingerprint": 1000000,
                      "delta_sync": null,
                      "change_address": "xch1dtfukqqka3ftqtdlhmc5spc5vd44h7ejrtnjcewxlueam5yrnnqqyczg8t"
                    }
                  ]
                }"#]],
        );
    }

    #[test]
    fn test_vault_wallet() {
        let config = Wallet {
            change_address: None,
            kind: WalletKind::Vault {
                launcher_id: Bytes32::new([1; 32]),
            },
            ..default()
        };
        check(
            config,
            &expect![[r#"
            version = 1

            [defaults]
            delta_sync = true

            [[wallets]]
            name = "Main"
            fingerprint = 1000000

            [wallets.kind]
            type = "vault"
            launcher_id = "0101010101010101010101010101010101010101010101010101010101010101"
        "#]],
            &expect![[r#"
            {
              "version": 1,
              "defaults": {
                "delta_sync": true
              },
              "wallets": [
                {
                  "name": "Main",
                  "fingerprint": 1000000,
                  "delta_sync": null,
                  "kind": {
                    "type": "vault",
                    "launcher_id": "0101010101010101010101010101010101010101010101010101010101010101"
                  }
                }
              ]
            }"#]],
        );
    }

    #[test]
    fn test_watch_wallet() {
        let config = Wallet {
            change_address: None,
            kind: WalletKind::Watch {
                p2_puzzle_hashes: vec![Bytes32::new([2; 32]), Bytes32::new([3; 32])],
            },
            ..default()
        };
        check(
            config,
            &expect![[r#"
            version = 1

            [defaults]
            delta_sync = true

            [[wallets]]
            name = "Main"
            fingerprint = 1000000

            [wallets.kind]
            type = "watch"
            p2_puzzle_hashes = [
                "0202020202020202020202020202020202020202020202020202020202020202",
                "0303030303030303030303030303030303030303030303030303030303030303",
            ]
        "#]],
            &expect![[r#"
            {
              "version": 1,
              "defaults": {
                "delta_sync": true
              },
              "wallets": [
                {
                  "name": "Main",
                  "fingerprint": 1000000,
                  "delta_sync": null,
                  "kind": {
                    "type": "watch",
                    "p2_puzzle_hashes": [
                      "0202020202020202020202020202020202020202020202020202020202020202",
                      "0303030303030303030303030303030303030303030303030303030303030303"
                    ]
                  }
                }
              ]
            }"#]],
        );
    }

    #[test]
    fn test_unversioned_wallet_config() {
        let config: WalletConfig =
            toml::from_str("[[wallets]]\nname = \"Main\"\nfingerprint = 1000000\n")
                .expect("Failed to parse toml");

        assert_eq!(config.version, 0);
        assert_eq!(config.wallets[0].fingerprint, 1_000_000);
        assert_eq!(config.wallets[0].kind, WalletKind::Bls);
    }

    #[test]
    fn test_wallet_kind_round_trips() {
        let config = WalletConfig {
            version: WALLET_CONFIG_VERSION,
            defaults: WalletDefaults::default(),
            wallets: vec![
                default(),
                Wallet {
                    kind: WalletKind::Vault {
                        launcher_id: Bytes32::new([1; 32]),
                    },
                    ..default()
                },
                Wallet {
                    kind: WalletKind::Watch {
                        p2_puzzle_hashes: vec![Bytes32::new([2; 32])],
                    },
                    ..default()
                },
            ],
        };

        let toml = toml::to_string_pretty(&config).expect("Failed to serialize toml");
        let parsed: WalletConfig = toml::from_str(&toml).expect("Failed to parse toml");

        assert_eq!(parsed.version, WALLET_CONFIG_VERSION);
        let kinds: Vec<WalletKind> = parsed
            .wallets
            .into_iter()
            .map(|wallet| wallet.kind)
            .collect();
        let expected: Vec<WalletKind> = config
            .wallets
            .into_iter()
            .map(|wallet| wallet.kind)
            .collect();
        assert_eq!(kinds, expected);
    }
}

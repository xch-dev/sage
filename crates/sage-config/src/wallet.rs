use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Default, Clone, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct WalletConfig {
    pub defaults: WalletDefaults,
    pub wallets: Vec<Wallet>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
pub struct WalletDefaults {
    #[serde(default, flatten)]
    pub change_mode: ChangeMode,
    #[serde(default, flatten)]
    pub derivation_mode: DerivationMode,
}

impl Default for WalletDefaults {
    fn default() -> Self {
        Self {
            change_mode: ChangeMode::Same,
            derivation_mode: DerivationMode::Auto {
                derivation_batch_size: 1000,
            },
        }
    }
}
#[derive(Debug, Default, Clone, Serialize, Deserialize, Type)]
pub struct Wallet {
    #[serde(default = "default_name")]
    pub name: String,
    pub fingerprint: u32,
    #[serde(default, flatten, skip_serializing_if = "ChangeMode::is_default")]
    pub change_mode: ChangeMode,
    #[serde(default, flatten, skip_serializing_if = "DerivationMode::is_default")]
    pub derivation_mode: DerivationMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub network: Option<String>,
}

fn default_name() -> String {
    "Unnamed".to_string()
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case", tag = "change_mode")]
pub enum ChangeMode {
    #[default]
    Default,
    /// Reuse the first address of coins involved in the transaction
    /// as the change address for the output. This improves compatibility
    /// with wallets which do not support multiple addresses.
    Same,
    /// Use an address that has not been used before as the change address
    /// for the output. This is beneficial for privacy, but results in more
    /// addresses being generated and used which can make syncing slower.
    New,
}

impl ChangeMode {
    pub fn is_default(&self) -> bool {
        matches!(self, Self::Default)
    }
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case", tag = "derivation_mode")]
pub enum DerivationMode {
    #[default]
    Default,
    /// Automatically generate new addresses if there aren't enough that
    /// haven't been used yet.
    Auto {
        /// The number of addresses to generate at a time.
        derivation_batch_size: u32,
    },
    /// Don't generate any new addresses, only use existing ones.
    Static,
}

impl DerivationMode {
    pub fn is_default(&self) -> bool {
        matches!(self, Self::Default)
    }
}

#[cfg(test)]
mod tests {
    use expect_test::{expect, Expect};

    use super::*;

    fn default() -> Wallet {
        Wallet {
            fingerprint: 1_000_000,
            name: "Main".to_string(),
            change_mode: ChangeMode::Default,
            derivation_mode: DerivationMode::Default,
            network: None,
        }
    }

    fn check(value: Wallet, expect_toml: &Expect, expect_json: &Expect) {
        let value = WalletConfig {
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
                [defaults]
                change_mode = "same"
                derivation_mode = "auto"
                derivation_batch_size = 1000

                [[wallets]]
                name = "Main"
                fingerprint = 1000000
            "#]],
            &expect![[r#"
                {
                  "defaults": {
                    "change_mode": "same",
                    "derivation_mode": "auto",
                    "derivation_batch_size": 1000
                  },
                  "wallets": [
                    {
                      "name": "Main",
                      "fingerprint": 1000000
                    }
                  ]
                }"#]],
        );
    }

    #[test]
    fn test_wallet_config_override() {
        let config = Wallet {
            change_mode: ChangeMode::Same,
            derivation_mode: DerivationMode::Auto {
                derivation_batch_size: 1000,
            },
            ..default()
        };
        check(
            config,
            &expect![[r#"
                [defaults]
                change_mode = "same"
                derivation_mode = "auto"
                derivation_batch_size = 1000

                [[wallets]]
                name = "Main"
                fingerprint = 1000000
                change_mode = "same"
                derivation_mode = "auto"
                derivation_batch_size = 1000
            "#]],
            &expect![[r#"
                {
                  "defaults": {
                    "change_mode": "same",
                    "derivation_mode": "auto",
                    "derivation_batch_size": 1000
                  },
                  "wallets": [
                    {
                      "name": "Main",
                      "fingerprint": 1000000,
                      "change_mode": "same",
                      "derivation_mode": "auto",
                      "derivation_batch_size": 1000
                    }
                  ]
                }"#]],
        );
    }
}

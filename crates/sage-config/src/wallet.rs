use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Default, Clone, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct WalletConfig {
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
        }
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
            network: None,
            delta_sync: None,
            emoji: None,
            change_address: Some(
                "xch1dtfukqqka3ftqtdlhmc5spc5vd44h7ejrtnjcewxlueam5yrnnqqyczg8t".to_string(),
            ),
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
                delta_sync = true

                [[wallets]]
                name = "Main"
                fingerprint = 1000000
                change_address = "xch1dtfukqqka3ftqtdlhmc5spc5vd44h7ejrtnjcewxlueam5yrnnqqyczg8t"
            "#]],
            &expect![[r#"
                {
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
                [defaults]
                delta_sync = true

                [[wallets]]
                name = "Main"
                fingerprint = 1000000
                change_address = "xch1dtfukqqka3ftqtdlhmc5spc5vd44h7ejrtnjcewxlueam5yrnnqqyczg8t"
            "#]],
            &expect![[r#"
                {
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
}

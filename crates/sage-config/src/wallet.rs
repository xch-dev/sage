use serde::{Deserialize, Serialize};
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
    /// What the wallet is made of. It's stored alongside the wallet's own fields, under a `type`
    /// key, which is omitted for BLS wallets since every wallet was one before kinds existed.
    #[serde(flatten, skip_serializing_if = "is_default")]
    pub kind: WalletKind,
}

/// The kind of a wallet. A BLS wallet's key is stored in the keychain under the wallet's
/// fingerprint.
///
/// It's read through [`RawWalletKind`], since a flattened field can't have a default, and a
/// missing `type` means BLS.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(tag = "type", rename_all = "snake_case", from = "RawWalletKind")]
// Renamed so that it doesn't clash with the API's `WalletKind` in the TypeScript bindings.
#[specta(rename = "WalletConfigKind")]
pub enum WalletKind {
    #[default]
    Bls,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum WalletKindTag {
    Bls,
}

/// The fields of every kind of wallet, which are checked against the `type` when converting
/// to a [`WalletKind`].
#[derive(Debug, Deserialize)]
struct RawWalletKind {
    #[serde(rename = "type")]
    tag: Option<WalletKindTag>,
}

impl From<RawWalletKind> for WalletKind {
    fn from(raw: RawWalletKind) -> Self {
        match raw.tag.unwrap_or(WalletKindTag::Bls) {
            WalletKindTag::Bls => Self::Bls,
        }
    }
}

impl Wallet {
    pub fn delta_sync(&self, defaults: &WalletDefaults) -> bool {
        self.delta_sync.unwrap_or(defaults.delta_sync)
    }
}

// BLS wallets are the default kind, so their `type` is left out.
fn is_default<T: Default + PartialEq>(value: &T) -> bool {
    *value == T::default()
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
    fn test_unversioned_wallet_config() {
        let config: WalletConfig =
            toml::from_str("[[wallets]]\nname = \"Main\"\nfingerprint = 1000000\n")
                .expect("Failed to parse toml");

        assert_eq!(config.version, 0);
        assert_eq!(config.wallets[0].fingerprint, 1_000_000);
        assert_eq!(config.wallets[0].kind, WalletKind::Bls);
    }

    #[test]
    fn test_wallet_kinds_parse() {
        let config: WalletConfig = toml::from_str(
            "[[wallets]]\nname = \"Old\"\nfingerprint = 4000000000\nunknown = 1\n\n\
             [[wallets]]\nname = \"Explicit\"\nfingerprint = 2\ntype = \"bls\"\n",
        )
        .expect("Failed to parse toml");

        assert_eq!(config.wallets[0].fingerprint, 4_000_000_000);
        assert_eq!(config.wallets[0].kind, WalletKind::Bls);
        assert_eq!(config.wallets[1].kind, WalletKind::Bls);

        let result = toml::from_str::<WalletConfig>(
            "[[wallets]]\nname = \"Bad\"\nfingerprint = 1\ntype = \"other\"\n",
        );
        assert!(result.is_err());
    }
}

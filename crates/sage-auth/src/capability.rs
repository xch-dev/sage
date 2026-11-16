use serde::{Deserialize, Serialize};

macro_rules! define_capabilities {
    (
        $(
            $variant:ident => $key:literal $(| $alias:literal)*, $endpoint:literal,
            $label:literal, $description:literal;
        )*
    ) => {
        /// A permission a caller can be granted over the wallet.
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
        )]
        #[cfg_attr(feature = "tauri", derive(specta::Type))]
        pub enum Capability {
            $(
                #[serde(rename = $key $(, alias = $alias)*)]
                #[cfg_attr(feature = "tauri", specta(rename = $key))]
                $variant,
            )*
        }

        impl Capability {
            /// Every capability, in declaration order.
            pub const ALL: &'static [Capability] = &[$(Self::$variant),*];

            /// The stable string key, `wallet.` plus the endpoint name.
            pub fn key(self) -> &'static str {
                match self {
                    $(Self::$variant => $key),*
                }
            }

            /// Parses a key, including legacy aliases.
            pub fn from_key(key: &str) -> Option<Self> {
                match key {
                    $($key $(| $alias)* => Some(Self::$variant),)*
                    _ => None,
                }
            }

            /// The sage-api endpoint this capability gates, if it maps to one.
            pub fn endpoint(self) -> Option<&'static str> {
                match self {
                    $(Self::$variant => Some($endpoint)),*
                }
            }

            /// A short human-readable name.
            pub fn label(self) -> &'static str {
                match self {
                    $(Self::$variant => $label),*
                }
            }

            /// A one-sentence explanation of what the capability allows.
            pub fn description(self) -> &'static str {
                match self {
                    $(Self::$variant => $description),*
                }
            }
        }

        /// The capability that gates a sage-api endpoint; `None` if no grant can reach it.
        pub fn endpoint_capability(endpoint: &str) -> Option<Capability> {
            match endpoint {
                $($endpoint => Some(Capability::$variant),)*
                _ => None,
            }
        }
    };
}

define_capabilities! {
    WalletGetWallet => "wallet.get_wallet" | "wallet.get_key", "get_wallet",
        "Read wallet", "Allows the caller to read public information about the selected wallet.";
    WalletGetSyncStatus => "wallet.get_sync_status", "get_sync_status",
        "Read sync status", "Allows the caller to read wallet sync status and current wallet balance summary.";
    WalletGetVersion => "wallet.get_version", "get_version",
        "Read wallet version", "Allows the caller to read the current Sage wallet version.";
    WalletGetToken => "wallet.get_token", "get_token",
        "Read token", "Allows the caller to read details and balance of a token in the wallet.";
    WalletGetCoins => "wallet.get_coins", "get_coins",
        "Read coins", "Allows the caller to list wallet coins.";
    WalletGetCoinsByIds => "wallet.get_coins_by_ids", "get_coins_by_ids",
        "Read coins by IDs", "Allows the caller to read specific wallet coin records by coin ID.";
    WalletGetSpendableCoinCount => "wallet.get_spendable_coin_count", "get_spendable_coin_count",
        "Read spendable coin count", "Allows the caller to read the number of spendable coins in the wallet.";
    WalletGetTransactions => "wallet.get_transactions", "get_transactions",
        "Read transactions", "Allows the caller to list wallet transactions.";
    WalletGetTransaction => "wallet.get_transaction", "get_transaction",
        "Read transaction", "Allows the caller to read a wallet transaction by height.";
    WalletGetPendingTransactions => "wallet.get_pending_transactions", "get_pending_transactions",
        "Read pending transactions", "Allows the caller to read pending wallet transactions.";
    WalletGetXchUsdPrice => "wallet.get_xch_usd_price", "get_xch_usd_price",
        "Read XCH/USD price", "Allows the caller to read the current estimated XCH price in USD.";
    WalletCheckAddress => "wallet.check_address", "check_address",
        "Check address", "Allows the caller to validate whether an address belongs to this wallet.";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_are_wallet_dot_endpoint_and_round_trip() {
        for &c in Capability::ALL {
            let endpoint = c.endpoint().unwrap();
            assert_eq!(c.key(), format!("wallet.{endpoint}"));
            assert_eq!(Capability::from_key(c.key()), Some(c));
            assert_eq!(endpoint_capability(endpoint), Some(c));
            assert_eq!(serde_json::to_value(c).unwrap(), serde_json::json!(c.key()));
            assert!(!c.label().is_empty() && !c.description().is_empty());
        }
        assert_eq!(Capability::ALL.len(), 12);
    }

    #[test]
    fn old_apps_alias_still_parses() {
        assert_eq!(
            Capability::from_key("wallet.get_key"),
            Some(Capability::WalletGetWallet)
        );
        assert_eq!(
            serde_json::from_str::<Capability>("\"wallet.get_key\"").unwrap(),
            Capability::WalletGetWallet
        );
    }

    #[test]
    fn endpoints_are_real_sage_api_endpoints() {
        let endpoints: serde_json::Value =
            serde_json::from_str(include_str!("../../sage-api/endpoints.json")).unwrap();
        for &c in Capability::ALL {
            let e = c.endpoint().unwrap();
            assert!(endpoints.get(e).is_some(), "{e}");
        }
    }

    #[test]
    fn write_and_secret_endpoints_are_unmapped() {
        for e in [
            "send_xch",
            "sign_coin_spends",
            "get_secret_key",
            "delete_wallet",
            "login",
            "submit_transaction",
        ] {
            assert_eq!(endpoint_capability(e), None, "{e}");
        }
    }
}

use serde::{Deserialize, Serialize};
use specta::Type;

macro_rules! define_wallet_capabilities {
    (
        $visibility:vis enum $name:ident {
            $(
                $variant:ident => $key:literal $(| $alias:literal)*
            ),* $(,)?
        }
    ) => {
        #[derive(
            Debug,
            Clone,
            Copy,
            Serialize,
            Deserialize,
            Type,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Hash,
        )]
        $visibility enum $name {
            $(
                #[serde(rename = $key $(, alias = $alias)*)]
                #[specta(rename = $key)]
                $variant,
            )*
        }

        impl $name {
            pub const ALL: &'static [Self] = &[
                $(Self::$variant),*
            ];

            pub fn key(self) -> &'static str {
                match self {
                    $(Self::$variant => $key),*
                }
            }

            pub fn from_key(key: &str) -> Option<Self> {
                match key {
                    $($key $(| $alias)* => Some(Self::$variant),)*
                    _ => None,
                }
            }
        }
    };
}

define_wallet_capabilities! {
    pub enum WalletCapability {
        WalletGetWallet => "wallet.get_wallet" | "wallet.get_key",
        WalletGetSecretKey => "wallet.get_secret_key",
        WalletSendXch => "wallet.send_xch",
        WalletSendXchAutoSubmit => "wallet.send_xch_auto_submit",
        WalletGetSyncStatus => "wallet.get_sync_status",
        WalletGetVersion => "wallet.get_version",
        WalletGetXchUsdPrice => "wallet.get_xch_usd_price",
        WalletCheckAddress => "wallet.check_address",
        WalletResolveHandle => "wallet.resolve_handle",
        WalletGetHandles => "wallet.get_handles",
        WalletGetContacts => "wallet.get_contacts",
        WalletFilterUnlockedCoins => "wallet.filter_unlocked_coins",
        WalletGetAssetCoins => "wallet.get_asset_coins",
        WalletGetAssetBalance => "wallet.get_asset_balance",
        WalletSignCoinSpends => "wallet.sign_coin_spends",
        WalletSignMessage => "wallet.sign_message",
        WalletSendTransaction => "wallet.send_transaction",
        WalletGetPublicKeys => "wallet.get_public_keys",
        WalletGetDerivations => "wallet.get_derivations",
        WalletGetSpendableCoinCount => "wallet.get_spendable_coin_count",
        WalletGetCoinsByIds => "wallet.get_coins_by_ids",
        WalletGetCoins => "wallet.get_coins",
        WalletGetPendingTransactions => "wallet.get_pending_transactions",
        WalletGetTransaction => "wallet.get_transaction",
        WalletGetTransactions => "wallet.get_transactions",
    }
}

use std::collections::BTreeSet;
use serde::{Deserialize, Serialize};
use specta::Type;
use crate::permissions::capabilities::get_user_capability_definition;

macro_rules! define_bridge_capabilities {
    (
        $visibility:vis enum $name:ident {
            $(
                $variant:ident => $key:expr
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
                #[serde(rename = $key)]
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
                    $($key => Some(Self::$variant),)*
                    _ => None,
                }
            }
        }
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BridgeCapability {
    User(UserBridgeCapability),
    System(SystemBridgeCapability),
}

define_bridge_capabilities! {
    pub enum UserBridgeCapability {
        PersistentStorage => "persistent_storage",
        BridgeSend => "bridge.send",
        AppGetCapabilities => "app.get_capabilities",
        AppGetInfo => "app.get_info",
        AppLifecycleReadyToStop => "app.lifecycle.ready_to_stop",
        AppLifecycleSetBeforeStopListener => "app.lifecycle.set_before_stop_listener",
        AppRequestCapabilityGrant => "app.request_capability_grant",
        AppRequestNetworkWhitelistGrant => "app.request_network_whitelist_grant",
        WalletGetKeys => "wallet.get_keys",
        WalletGetKey => "wallet.get_key",
        WalletGetSecretKey => "wallet.get_secret_key",
        WalletSendXch => "wallet.send_xch",
        WalletSendXchAutoSubmit => "wallet.send_xch_auto_submit",
        WalletGetSyncStatus => "wallet.get_sync_status",
        WalletGetVersion => "wallet.get_version",
        WalletCheckAddress => "wallet.check_address",
        WalletGetDerivations => "wallet.get_derivations",
        WalletGetSpendableCoinCount => "wallet.get_spendable_coin_count",
        WalletGetCoinsByIds => "wallet.get_coins_by_ids",
        WalletGetCoins => "wallet.get_coins",
        WalletGetPendingTransactions => "wallet.get_pending_transactions",
        WalletGetTransaction => "wallet.get_transaction",
        WalletGetTransactions => "wallet.get_transactions",
    }
}

define_bridge_capabilities! {
    pub enum SystemBridgeCapability {
        RuntimeManagerListRuntimes => "runtime_manager.list_runtimes",
        RuntimeManagerFocusRuntime => "runtime_manager.focus_runtime",
        RuntimeManagerHideRuntime => "runtime_manager.hide_runtime",
        RuntimeManagerKillRuntime => "runtime_manager.kill_runtime",
        RuntimeManagerListenRuntimesChanged => "runtime_manager.listen_runtimes_changed",
    }
}

impl BridgeCapability {
    pub fn key(self) -> &'static str {
        match self {
            Self::User(capability) => capability.key(),
            Self::System(capability) => capability.key(),
        }
    }
}

impl UserBridgeCapability {
    pub fn shared_from_granted(
        granted: &[UserBridgeCapability],
    ) -> Vec<UserBridgeCapability> {
        let mut shared = BTreeSet::new();

        for capability in granted {
            let definition = get_user_capability_definition(*capability);

            if definition.flags.shared_with_app {
                shared.insert(*capability);
            }
        }

        shared.into_iter().collect()
    }
}

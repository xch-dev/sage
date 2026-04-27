use crate::permissions::get_user_capability_definition;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::BTreeSet;

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

pub trait SharedCapabilitiesExt {
    fn shared(self) -> Vec<UserBridgeCapability>;
}

impl<I> SharedCapabilitiesExt for I
where
    I: IntoIterator<Item = UserBridgeCapability>,
{
    fn shared(self) -> Vec<UserBridgeCapability> {
        self.into_iter()
            .filter(|cap| get_user_capability_definition(*cap).flags.shared_with_app)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::bridge::capabilities::{SharedCapabilitiesExt, UserBridgeCapability};
    use crate::permissions::user_registry;

    fn first_shared_capability() -> UserBridgeCapability {
        user_registry()
            .values()
            .find(|definition| definition.flags.shared_with_app)
            .unwrap_or_else(|| {
                panic!("test requires at least one capability with shared_with_app = true")
            })
            .capability
    }

    fn first_non_shared_capability() -> UserBridgeCapability {
        user_registry()
            .values()
            .find(|definition| !definition.flags.shared_with_app)
            .unwrap_or_else(|| {
                panic!("test requires at least one capability with shared_with_app = false")
            })
            .capability
    }

    #[test]
    fn resolve_shared_capabilities_filters_out_non_shared_capabilities() {
        let shared = first_shared_capability();
        let non_shared = first_non_shared_capability();

        let shared_capabilities = [shared, non_shared].shared();

        assert!(
            shared_capabilities.contains(&shared),
            "shared capability should remain visible to app"
        );
        assert!(
            !shared_capabilities.contains(&non_shared),
            "non-shared capability should not be visible to app"
        );
    }

    #[test]
    fn resolve_shared_capabilities_preserves_ordered_unique_shared_subset() {
        let shared = first_shared_capability();
        let non_shared = first_non_shared_capability();

        let shared_capabilities = [non_shared, shared, shared].shared();

        assert_eq!(shared_capabilities, vec![shared]);
    }
}

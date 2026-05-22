use crate::capabilities::get_user_capability_definition;
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
        BridgeSend => "bridge.send",

        AppGetInfo => "app.get_info",

        AppLifecycleReadyToStop => "app.lifecycle.ready_to_stop",
        AppLifecycleSetBeforeStopListener => "app.lifecycle.set_before_stop_listener",

        AppGetCapabilities => "app.get_capabilities",
        AppRequestCapabilityGrant => "app.request_capability_grant",
        AppRequestNetworkWhitelistGrant => "app.request_network_whitelist_grant",

        WalletGetKey => "wallet.get_key",
        WalletGetSecretKey => "wallet.get_secret_key",
        WalletSendXch => "wallet.send_xch",
        WalletSendXchAutoSubmit => "wallet.send_xch_auto_submit",
        WalletGetSyncStatus => "wallet.get_sync_status",
        WalletGetVersion => "wallet.get_version",
        WalletGetXchUsdPrice => "wallet.get_xch_usd_price",
        WalletCheckAddress => "wallet.check_address",
        WalletGetDerivations => "wallet.get_derivations",
        WalletGetSpendableCoinCount => "wallet.get_spendable_coin_count",
        WalletGetCoinsByIds => "wallet.get_coins_by_ids",
        WalletGetCoins => "wallet.get_coins",
        WalletGetPendingTransactions => "wallet.get_pending_transactions",
        WalletGetTransaction => "wallet.get_transaction",
        WalletGetTransactions => "wallet.get_transactions",

        EnvironmentThemeGetCurrent => "environment.theme.get_current",
        EnvironmentThemeCssVars => "environment.theme.css_vars",
        EnvironmentThemeListenChanged => "environment.theme.listen_changed",

        StoragePersistentWebview => "storage.persistent_webview",
    }
}

define_bridge_capabilities! {
    pub enum SystemBridgeCapability {
        RuntimeManagerListRuntimes => "runtime_manager.list_runtimes",
        RuntimeManagerFocusTaskbarRuntime => "runtime_manager.focus_taskbar_runtime",
        RuntimeManagerHideRuntime => "runtime_manager.hide_runtime",
        RuntimeManagerKillRuntime => "runtime_manager.kill_runtime",
        RuntimeManagerGetActiveTaskbarRuntime => "runtime_manager.get_active_taskbar_runtime",
        RuntimeManagerListenRuntimesChanged => "runtime_manager.listen_runtimes_changed",
        RuntimeManagerListenActiveTaskbarRuntimeChanged => "runtime_manager.listen_active_runtime_changed",
        RuntimeManagerHideSelf => "runtime_manager.hide_self",
        RuntimeManagerCloseSelf => "runtime_manager.close_self",

        CapabilityDefinitionsRead => "capability_definitions.read",

        AppPermissionsRead => "app_permissions.read",
        AppPermissionsApply => "app_permissions.apply",

        AppInstallPreview => "app_install.preview",
        AppInstallApply => "app_install.apply",

        AppUpdateRead => "app_update.read",
        AppUpdateApply => "app_update.apply",

        AppRegistryListenListedAppsChanged => "app_registry.listen_listed_apps_changed",

        FileSystemSelectFile => "file_system.select_file",

        BridgeApprovalList => "bridge_approval.list",
        BridgeApprovalResolve => "bridge_approval.resolve",
        BridgeApprovalListenApprovalsChanged => "bridge_approval.listen_changed",

        DonationGetDetails => "donation.get_details",

        SandboxGetState => "sandbox.get_state",
        SandboxRerunTests => "sandbox.rerun_tests",
        SandboxListenStateChanged => "sandbox.listen_state_changed",

        WalletListWallets => "wallet.list_wallets",
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
            .filter(|cap| {
                get_user_capability_definition(*cap)
                    .flags()
                    .shared_with_app()
            })
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }
}

impl From<UserBridgeCapability> for BridgeCapability {
    fn from(value: UserBridgeCapability) -> Self {
        Self::User(value)
    }
}

impl From<SystemBridgeCapability> for BridgeCapability {
    fn from(value: SystemBridgeCapability) -> Self {
        Self::System(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::capabilities::list::{SharedCapabilitiesExt, UserBridgeCapability};
    use crate::capabilities::user_registry;

    fn first_shared_capability() -> UserBridgeCapability {
        user_registry()
            .values()
            .find(|definition| definition.flags().shared_with_app())
            .unwrap_or_else(|| {
                panic!("test requires at least one capability with shared_with_app = true")
            })
            .capability()
    }

    fn first_non_shared_capability() -> UserBridgeCapability {
        user_registry()
            .values()
            .find(|definition| !definition.flags().shared_with_app())
            .unwrap_or_else(|| {
                panic!("test requires at least one capability with shared_with_app = false")
            })
            .capability()
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

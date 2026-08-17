use std::collections::HashMap;

use crate::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BridgeRegistryKind {
    User,
    System,
}

pub(crate) struct BridgeRegistry {
    methods: HashMap<&'static str, Box<dyn BridgeMethod>>,
}

impl BridgeRegistry {
    pub(crate) fn new(kind: BridgeRegistryKind) -> Self {
        match kind {
            BridgeRegistryKind::User => Self {
                methods: build_user_methods(),
            },
            BridgeRegistryKind::System => Self {
                methods: build_system_methods(),
            },
        }
    }

    pub(crate) fn get(&self, method: &str) -> Option<&dyn BridgeMethod> {
        self.methods.get(method).map(AsRef::as_ref)
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (&'static str, &dyn BridgeMethod)> {
        self.methods
            .iter()
            .map(|(name, method)| (*name, method.as_ref()))
    }
}

fn build_user_methods() -> HashMap<&'static str, Box<dyn BridgeMethod>> {
    let mut methods: HashMap<&'static str, Box<dyn BridgeMethod>> = HashMap::new();

    // Bridge
    insert_method(&mut methods, BridgePing);
    insert_method(&mut methods, BridgeSend);

    // App
    insert_method(&mut methods, AppGetInfo);
    insert_method(&mut methods, AppGetCapabilities);
    insert_method(&mut methods, AppRequestCapabilityGrant);
    insert_method(&mut methods, AppRequestNetworkWhitelistGrant);
    insert_method(&mut methods, AppLifecycleSetBeforeStopListener);
    insert_method(&mut methods, AppLifecycleReadyToStop);

    // Wallet keys / secrets
    insert_method(&mut methods, WalletGetKey);
    insert_method(&mut methods, WalletGetSecretKey);
    insert_method(&mut methods, WalletGetPublicKeys);

    // Wallet XCH
    insert_method(&mut methods, WalletSendXch);

    // Wallet read/query
    insert_method(&mut methods, WalletGetSyncStatus);
    insert_method(&mut methods, WalletGetVersion);
    insert_method(&mut methods, WalletGetPendingTransactions);
    insert_method(&mut methods, WalletGetXchUsdPrice);
    insert_method(&mut methods, WalletCheckAddress);
    insert_method(&mut methods, WalletGetDerivations);
    insert_method(&mut methods, WalletGetSpendableCoinCount);
    insert_method(&mut methods, WalletGetCoinsByIds);
    insert_method(&mut methods, WalletGetCoins);
    insert_method(&mut methods, WalletGetTransaction);
    insert_method(&mut methods, WalletGetTransactions);

    // Environment
    insert_method(&mut methods, EnvironmentThemeGetCurrent);
    insert_method(&mut methods, EnvironmentGetNetwork);

    methods
}

fn build_system_methods() -> HashMap<&'static str, Box<dyn BridgeMethod>> {
    let mut methods: HashMap<&'static str, Box<dyn BridgeMethod>> = HashMap::new();

    insert_method(&mut methods, RuntimeManagerListRuntimes);
    insert_method(&mut methods, RuntimeManagerFocusTaskbarRuntime);
    insert_method(&mut methods, RuntimeManagerHideRuntime);
    insert_method(&mut methods, RuntimeManagerKillRuntime);
    insert_method(&mut methods, RuntimeManagerHideSelf);
    insert_method(&mut methods, RuntimeManagerCloseSelf);
    insert_method(&mut methods, RuntimeManagerGetActiveTaskbarRuntime);

    insert_method(&mut methods, AppInstallPreviewUrl);
    insert_method(&mut methods, AppInstallPreviewZip);
    insert_method(&mut methods, AppInstallInstallUrl);
    insert_method(&mut methods, AppInstallInstallZip);

    insert_method(&mut methods, AppUpdateGetReviewContext);
    insert_method(&mut methods, AppUpdateApplyUpdate);

    insert_method(&mut methods, CapabilitiesListUserDefinitions);
    insert_method(&mut methods, AppPermissionsGetReviewContext);
    insert_method(&mut methods, AppPermissionsApplyPermissions);

    insert_method(&mut methods, FileSystemSelectFile);

    insert_method(&mut methods, BridgeApprovalsListPending);
    insert_method(&mut methods, BridgeApprovalsResolve);

    insert_method(&mut methods, DonationGetDetails);

    insert_method(&mut methods, SandboxGetState);
    insert_method(&mut methods, SandboxRerunTests);

    insert_method(&mut methods, WalletListWallets);

    methods
}

impl std::fmt::Debug for BridgeRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BridgeRegistry")
            .field("method_count", &self.methods.len())
            .finish()
    }
}

fn insert_method<M>(methods: &mut HashMap<&'static str, Box<dyn BridgeMethod>>, method: M)
where
    M: BridgeMethod + 'static,
{
    methods.insert(method.name(), Box::new(method));
}

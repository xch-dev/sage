use std::collections::HashMap;

use crate::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BridgeRegistryKind {
    User,
    System,
}

pub(crate) enum BridgeRegistry {
    User(UserBridgeRegistry),
    System(SystemBridgeRegistry),
}

pub(crate) struct UserBridgeRegistry {
    methods: HashMap<&'static str, UserBridgeMethodEntry>,
}

pub(crate) struct SystemBridgeRegistry {
    methods: HashMap<&'static str, SystemBridgeMethodEntry>,
}

#[derive(Clone, Copy)]
pub(crate) enum BridgeMethodEntry<'a> {
    User(&'a UserBridgeMethodEntry),
    System(&'a SystemBridgeMethodEntry),
}

impl BridgeRegistry {
    pub(crate) fn new(kind: BridgeRegistryKind) -> Self {
        match kind {
            BridgeRegistryKind::User => Self::User(UserBridgeRegistry::new()),
            BridgeRegistryKind::System => Self::System(SystemBridgeRegistry::new()),
        }
    }

    pub(crate) fn get(&self, method: &str) -> Option<BridgeMethodEntry<'_>> {
        match self {
            Self::User(registry) => registry.get(method).map(BridgeMethodEntry::User),
            Self::System(registry) => registry.get(method).map(BridgeMethodEntry::System),
        }
    }

    pub(crate) fn iter(&self) -> Vec<(&'static str, BridgeMethodEntry<'_>)> {
        match self {
            Self::User(registry) => registry.iter(),
            Self::System(registry) => registry.iter(),
        }
    }
}

impl std::fmt::Debug for BridgeRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let method_count = match self {
            Self::User(registry) => registry.methods.len(),
            Self::System(registry) => registry.methods.len(),
        };

        f.debug_struct("BridgeRegistry")
            .field("method_count", &method_count)
            .finish()
    }
}

macro_rules! bridge_method_entry {
    ($entry:ident { $($variant:ident($method:ty)),+ $(,)? }) => {
        pub(crate) enum $entry {
            $($variant($method),)+
        }

        impl $entry {
            pub(crate) fn name(&self) -> &'static str {
                match self {
                    $(Self::$variant(method) => method.name(),)+
                }
            }

            pub(crate) fn capability(&self) -> BridgeMethodCapability {
                match self {
                    $(Self::$variant(method) => method.capability(),)+
                }
            }

            pub(crate) fn approval_request(
                &self,
                ctx: BridgeContext<'_>,
                request: &RustBridgeRequest,
            ) -> BridgeApprovalRequestResult {
                match self {
                    $(Self::$variant(method) => method.approval_request(ctx, request),)+
                }
            }

            pub(crate) fn handle(
                &self,
                ctx: BridgeContext<'_>,
                tools: BridgeTools<'_>,
                request: &RustBridgeRequest,
            ) -> impl std::future::Future<Output = BridgeHandleResult> + Send {
                async move {
                    match self {
                        $(Self::$variant(method) => method.handle(ctx, tools, request).await,)+
                    }
                }
            }
        }

        $(
            impl From<$method> for $entry {
                fn from(method: $method) -> Self {
                    Self::$variant(method)
                }
            }
        )+
    };
}

bridge_method_entry! { UserBridgeMethodEntry {
    BridgePingEntry(BridgePing),
    BridgeSendEntry(BridgeSend),
    AppGetInfoEntry(AppGetInfo),
    AppGetCapabilitiesEntry(AppGetCapabilities),
    AppRequestCapabilityGrantEntry(AppRequestCapabilityGrant),
    AppRequestNetworkWhitelistGrantEntry(AppRequestNetworkWhitelistGrant),
    AppLifecycleSetBeforeStopListenerEntry(AppLifecycleSetBeforeStopListener),
    AppLifecycleReadyToStopEntry(AppLifecycleReadyToStop),
    WalletGetKeyEntry(WalletGetKey),
    WalletGetSecretKeyEntry(WalletGetSecretKey),
    WalletSendXchEntry(WalletSendXch),
    WalletGetSyncStatusEntry(WalletGetSyncStatus),
    WalletGetVersionEntry(WalletGetVersion),
    WalletGetPendingTransactionsEntry(WalletGetPendingTransactions),
    WalletGetXchUsdPriceEntry(WalletGetXchUsdPrice),
    WalletCheckAddressEntry(WalletCheckAddress),
    WalletGetDerivationsEntry(WalletGetDerivations),
    WalletGetSpendableCoinCountEntry(WalletGetSpendableCoinCount),
    WalletGetCoinsByIdsEntry(WalletGetCoinsByIds),
    WalletGetCoinsEntry(WalletGetCoins),
    WalletGetTransactionEntry(WalletGetTransaction),
    WalletGetTransactionsEntry(WalletGetTransactions),
    EnvironmentThemeGetCurrentEntry(EnvironmentThemeGetCurrent),
    EnvironmentGetNetworkEntry(EnvironmentGetNetwork),
} }

bridge_method_entry! { SystemBridgeMethodEntry {
    RuntimeManagerListRuntimesEntry(RuntimeManagerListRuntimes),
    RuntimeManagerFocusTaskbarRuntimeEntry(RuntimeManagerFocusTaskbarRuntime),
    RuntimeManagerHideRuntimeEntry(RuntimeManagerHideRuntime),
    RuntimeManagerKillRuntimeEntry(RuntimeManagerKillRuntime),
    RuntimeManagerHideSelfEntry(RuntimeManagerHideSelf),
    RuntimeManagerCloseSelfEntry(RuntimeManagerCloseSelf),
    RuntimeManagerGetActiveTaskbarRuntimeEntry(RuntimeManagerGetActiveTaskbarRuntime),
    AppInstallPreviewUrlEntry(AppInstallPreviewUrl),
    AppInstallPreviewZipEntry(AppInstallPreviewZip),
    AppInstallInstallUrlEntry(AppInstallInstallUrl),
    AppInstallInstallZipEntry(AppInstallInstallZip),
    AppUpdateGetReviewContextEntry(AppUpdateGetReviewContext),
    AppUpdateApplyUpdateEntry(AppUpdateApplyUpdate),
    CapabilitiesListUserDefinitionsEntry(CapabilitiesListUserDefinitions),
    AppPermissionsGetReviewContextEntry(AppPermissionsGetReviewContext),
    AppPermissionsApplyPermissionsEntry(AppPermissionsApplyPermissions),
    FileSystemSelectFileEntry(FileSystemSelectFile),
    BridgeApprovalsListPendingEntry(BridgeApprovalsListPending),
    BridgeApprovalsResolveEntry(BridgeApprovalsResolve),
    DonationGetDetailsEntry(DonationGetDetails),
    SandboxGetStateEntry(SandboxGetState),
    SandboxRerunTestsEntry(SandboxRerunTests),
    WalletListWalletsEntry(WalletListWallets),
} }

impl UserBridgeRegistry {
    pub(crate) fn new() -> Self {
        let mut methods = HashMap::new();

        // Bridge
        insert_user_method(&mut methods, BridgePing);
        insert_user_method(&mut methods, BridgeSend);

        // App
        insert_user_method(&mut methods, AppGetInfo);
        insert_user_method(&mut methods, AppGetCapabilities);
        insert_user_method(&mut methods, AppRequestCapabilityGrant);
        insert_user_method(&mut methods, AppRequestNetworkWhitelistGrant);
        insert_user_method(&mut methods, AppLifecycleSetBeforeStopListener);
        insert_user_method(&mut methods, AppLifecycleReadyToStop);

        // Wallet keys / secrets
        insert_user_method(&mut methods, WalletGetKey);
        insert_user_method(&mut methods, WalletGetSecretKey);

        // Wallet XCH
        insert_user_method(&mut methods, WalletSendXch);

        // Wallet read/query
        insert_user_method(&mut methods, WalletGetSyncStatus);
        insert_user_method(&mut methods, WalletGetVersion);
        insert_user_method(&mut methods, WalletGetPendingTransactions);
        insert_user_method(&mut methods, WalletGetXchUsdPrice);
        insert_user_method(&mut methods, WalletCheckAddress);
        insert_user_method(&mut methods, WalletGetDerivations);
        insert_user_method(&mut methods, WalletGetSpendableCoinCount);
        insert_user_method(&mut methods, WalletGetCoinsByIds);
        insert_user_method(&mut methods, WalletGetCoins);
        insert_user_method(&mut methods, WalletGetTransaction);
        insert_user_method(&mut methods, WalletGetTransactions);

        // Environment
        insert_user_method(&mut methods, EnvironmentThemeGetCurrent);
        insert_user_method(&mut methods, EnvironmentGetNetwork);

        Self { methods }
    }

    pub(crate) fn get(&self, method: &str) -> Option<&UserBridgeMethodEntry> {
        self.methods.get(method)
    }

    fn iter(&self) -> Vec<(&'static str, BridgeMethodEntry<'_>)> {
        self.methods
            .iter()
            .map(|(name, method)| (*name, BridgeMethodEntry::User(method)))
            .collect()
    }
}

impl SystemBridgeRegistry {
    fn new() -> Self {
        let mut methods = HashMap::new();

        insert_system_method(&mut methods, RuntimeManagerListRuntimes);
        insert_system_method(&mut methods, RuntimeManagerFocusTaskbarRuntime);
        insert_system_method(&mut methods, RuntimeManagerHideRuntime);
        insert_system_method(&mut methods, RuntimeManagerKillRuntime);
        insert_system_method(&mut methods, RuntimeManagerHideSelf);
        insert_system_method(&mut methods, RuntimeManagerCloseSelf);
        insert_system_method(&mut methods, RuntimeManagerGetActiveTaskbarRuntime);

        insert_system_method(&mut methods, AppInstallPreviewUrl);
        insert_system_method(&mut methods, AppInstallPreviewZip);
        insert_system_method(&mut methods, AppInstallInstallUrl);
        insert_system_method(&mut methods, AppInstallInstallZip);

        insert_system_method(&mut methods, AppUpdateGetReviewContext);
        insert_system_method(&mut methods, AppUpdateApplyUpdate);

        insert_system_method(&mut methods, CapabilitiesListUserDefinitions);
        insert_system_method(&mut methods, AppPermissionsGetReviewContext);
        insert_system_method(&mut methods, AppPermissionsApplyPermissions);

        insert_system_method(&mut methods, FileSystemSelectFile);

        insert_system_method(&mut methods, BridgeApprovalsListPending);
        insert_system_method(&mut methods, BridgeApprovalsResolve);

        insert_system_method(&mut methods, DonationGetDetails);

        insert_system_method(&mut methods, SandboxGetState);
        insert_system_method(&mut methods, SandboxRerunTests);

        insert_system_method(&mut methods, WalletListWallets);

        Self { methods }
    }

    fn get(&self, method: &str) -> Option<&SystemBridgeMethodEntry> {
        self.methods.get(method)
    }

    fn iter(&self) -> Vec<(&'static str, BridgeMethodEntry<'_>)> {
        self.methods
            .iter()
            .map(|(name, method)| (*name, BridgeMethodEntry::System(method)))
            .collect()
    }
}

impl BridgeMethodEntry<'_> {
    pub(crate) fn capability(&self) -> BridgeMethodCapability {
        match self {
            Self::User(method) => method.capability(),
            Self::System(method) => method.capability(),
        }
    }

    pub(crate) fn approval_request(
        &self,
        ctx: BridgeContext<'_>,
        request: &RustBridgeRequest,
    ) -> BridgeApprovalRequestResult {
        match self {
            Self::User(method) => method.approval_request(ctx, request),
            Self::System(method) => method.approval_request(ctx, request),
        }
    }

    pub(crate) fn handle(
        &self,
        ctx: BridgeContext<'_>,
        tools: BridgeTools<'_>,
        request: &RustBridgeRequest,
    ) -> impl std::future::Future<Output = BridgeHandleResult> + Send {
        async move {
            match self {
                Self::User(method) => method.handle(ctx, tools, request).await,
                Self::System(method) => method.handle(ctx, tools, request).await,
            }
        }
    }
}

fn insert_user_method<M>(methods: &mut HashMap<&'static str, UserBridgeMethodEntry>, method: M)
where
    M: BridgeMethodHandler + Into<UserBridgeMethodEntry>,
{
    let method: UserBridgeMethodEntry = method.into();
    methods.insert(method.name(), method);
}

fn insert_system_method<M>(methods: &mut HashMap<&'static str, SystemBridgeMethodEntry>, method: M)
where
    M: BridgeMethodHandler + Into<SystemBridgeMethodEntry>,
{
    let method: SystemBridgeMethodEntry = method.into();
    methods.insert(method.name(), method);
}

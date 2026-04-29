use super::methods::BridgeMethod;
use super::methods::system::runtime_manager::{
    RuntimeManagerFocusRuntime, RuntimeManagerHideRuntime, RuntimeManagerKillRuntime,
    RuntimeManagerListRuntimes,
};
use super::methods::user::{
    AppGetCapabilities, AppGetInfo, AppLifecycleReadyToStop, AppLifecycleSetBeforeStopListener,
    AppRequestCapabilityGrant, AppRequestNetworkWhitelistGrant, BridgePing, BridgeSend,
    WalletCheckAddress, WalletGetCoins, WalletGetCoinsByIds, WalletGetDerivations, WalletGetKey,
    WalletGetKeys, WalletGetPendingTransactions, WalletGetSecretKey, WalletGetSpendableCoinCount,
    WalletGetSyncStatus, WalletGetTransaction, WalletGetTransactions, WalletGetVersion,
    WalletSendXch,
};
use crate::types::SharedSageApp;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeRegistryKind {
    User,
    System,
}

pub struct BridgeRegistry {
    methods: HashMap<&'static str, Box<dyn BridgeMethod>>,
}

impl BridgeRegistry {
    pub fn new(kind: BridgeRegistryKind) -> Self {
        match kind {
            BridgeRegistryKind::User => Self {
                methods: build_user_methods(),
            },
            BridgeRegistryKind::System => Self {
                methods: build_system_methods(),
            },
        }
    }

    pub fn new_for_app(app: &SharedSageApp) -> Self {
        if app.is_system_app() {
            return Self::new(BridgeRegistryKind::System);
        }

        Self::new(BridgeRegistryKind::User)
    }

    pub fn get(&self, method: &str) -> Option<&dyn BridgeMethod> {
        self.methods.get(method).map(std::convert::AsRef::as_ref)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&'static str, &dyn BridgeMethod)> {
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
    insert_method(&mut methods, WalletGetKeys);
    insert_method(&mut methods, WalletGetKey);
    insert_method(&mut methods, WalletGetSecretKey);

    // Wallet XCH
    insert_method(&mut methods, WalletSendXch);

    // Wallet read/query
    insert_method(&mut methods, WalletGetSyncStatus);
    insert_method(&mut methods, WalletGetVersion);
    insert_method(&mut methods, WalletGetPendingTransactions);
    insert_method(&mut methods, WalletCheckAddress);
    insert_method(&mut methods, WalletGetDerivations);
    insert_method(&mut methods, WalletGetSpendableCoinCount);
    insert_method(&mut methods, WalletGetCoinsByIds);
    insert_method(&mut methods, WalletGetCoins);
    insert_method(&mut methods, WalletGetTransaction);
    insert_method(&mut methods, WalletGetTransactions);

    methods
}

fn build_system_methods() -> HashMap<&'static str, Box<dyn BridgeMethod>> {
    let mut methods = build_user_methods();

    insert_method(&mut methods, RuntimeManagerListRuntimes);
    insert_method(&mut methods, RuntimeManagerFocusRuntime);
    insert_method(&mut methods, RuntimeManagerHideRuntime);
    insert_method(&mut methods, RuntimeManagerKillRuntime);

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

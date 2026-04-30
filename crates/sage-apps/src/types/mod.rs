mod app;
mod invariants;
mod manifest;
mod network;
mod normalizers;
mod permissions;
mod storage;
mod url;
mod view;

pub use app::SharedSageApp;

pub(crate) use app::{
    CorruptedInstalledSageApp, ListedSageApp, RetiredAppOriginEntry, SageApp, SageAppCommon,
    SageAppIdentity, SageAppSnapshot, SageAppUrlPreview, SystemAppPresentation, SystemSageApp,
    UserSageApp, UserSageAppPendingUpdate, UserSageAppSource, SageAppView, ResolvedApp, ResolvedStoppedApp, ResolvedRunningApp,
    ListedSageAppView, UserSageAppView,
};
pub(crate) use manifest::{SageAppManifestFile, SageAppPackageManifest};
pub(crate) use network::SageNetworkWhitelistEntry;
pub(crate) use permissions::{
    SageGrantedPermissions, SageGrantedSystemPermissions,
    SageRequestedPermissions,
};
pub(crate) use storage::{
    InstalledSageAppStorage, PendingStorageCleanupEntry, PendingStorageCleanupTarget,
};
pub(crate) use view::{
    SageAppCapabilityDefinitionView,
    SageGrantedPermissionsInput,
};

#[cfg(test)]
pub(crate) use manifest::SageAppPackageManifestParts;
#[cfg(test)]
pub(crate) use permissions::{SageRequestedCapabilities, SageRequestedNetworkPermissions};
pub(crate) use url::{MANIFEST_FILE_NAME, SageAppManifestUrl, SageAppUrl};

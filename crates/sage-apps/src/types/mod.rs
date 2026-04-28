mod app;
mod invariants;
mod manifest;
mod network;
mod normalizers;
mod permissions;
mod storage;
mod url;

pub use app::SageApp;

pub(crate) use app::{
    CorruptedInstalledSageApp, ListedSageApp, RetiredAppOriginEntry, SageAppCommon,
    SageAppIdentity, SageAppSnapshot, SageAppUrlPreview, SystemAppPresentation, SystemSageApp,
    UserSageApp, UserSageAppPendingUpdate, UserSageAppSource,
};
pub(crate) use manifest::{SageAppManifestFile, SageAppPackageManifest};
pub(crate) use network::SageNetworkWhitelistEntry;
pub(crate) use permissions::{
    SageAppCapabilityDefinitionView, SageGrantedPermissions, SageGrantedSystemPermissions,
    SageRequestedPermissions,
};
pub(crate) use storage::{
    InstalledSageAppStorage, PendingStorageCleanupEntry, PendingStorageCleanupTarget,
};

#[cfg(test)]
pub(crate) use manifest::SageAppPackageManifestParts;
#[cfg(test)]
pub(crate) use permissions::{SageRequestedCapabilities, SageRequestedNetworkPermissions};
pub(crate) use url::{MANIFEST_FILE_NAME, SageAppManifestUrl, SageAppUrl};

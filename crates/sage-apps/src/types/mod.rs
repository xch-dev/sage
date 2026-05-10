mod app;
mod invariants;
mod manifest;
mod network;
mod normalizers;
mod permissions;
mod storage;
mod url;

pub use app::SharedSageApp;

pub(crate) use app::{
    AppModalPresentation, AppPresentation, CorruptedInstalledSageApp, ListedSageApp, ResolvedApp,
    ResolvedRunningApp, ResolvedStoppedApp, RetiredAppOriginEntry, SageApp, SageAppCommon,
    SageAppIdentity, SageAppSnapshot, SageAppUrlPreview, SageAppWalletScope, SystemSageApp,
    UserSageApp, UserSageAppPendingUpdate, UserSageAppSource, view::*,
};
pub(crate) use manifest::{
    SageAppManifestFile, SageAppManifestHeaderV0, SageAppPackageManifest,
    SageAppPackageManifestPreview, parse_manifest_header_v0_from_value,
};
pub(crate) use network::SageNetworkWhitelistEntry;
pub(crate) use permissions::{
    SageGrantedPermissions, SageGrantedSystemPermissions, SageRequestedPermissions,
};
pub(crate) use storage::{
    InstalledSageAppStorage, PendingStorageCleanupEntry, PendingStorageCleanupTarget,
};

#[cfg(test)]
pub(crate) use manifest::{
    SageAppManifestSageVersion, SageAppManifestVersion, SageAppPackageManifestParts,
};
#[cfg(test)]
pub(crate) use permissions::{SageRequestedCapabilities, SageRequestedNetworkPermissions};
pub(crate) use url::{MANIFEST_FILE_NAME, SageAppManifestUrl, SageAppUrl};

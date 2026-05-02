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
    ListedSageAppView, UserSageAppView, SageAppIconView
};
pub(crate) use manifest::{
    SageAppManifestFile, SageAppManifestHeaderV0,
    SageAppPackageManifest,
    parse_manifest_header_v0_from_value,
};
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
pub(crate) use manifest::{SageAppPackageManifestParts, SageAppManifestSageVersion, SageAppManifestVersion};
#[cfg(test)]
pub(crate) use permissions::{SageRequestedCapabilities, SageRequestedNetworkPermissions};
pub(crate) use url::{MANIFEST_FILE_NAME, SageAppManifestUrl, SageAppUrl};

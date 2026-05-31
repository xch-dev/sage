use std::path::PathBuf;

use serde::{Deserialize, Deserializer, Serialize};

use crate::{
    CapabilityFlags,
    normalized_non_empty_string,
    SageAppPackageManifest,
    SageAppSnapshot,
    SageAppStorage,
    SageAppWalletScope,
    SageGrantedPermissions,
    SageRequestedPermissions,
    SANDBOX_TEST_ID_PREFIX,
    UserBridgeCapability,
    UserSageAppPendingUpdate,
    validate_snapshot_entry_and_icon_exist,
};

#[derive(Debug, Clone, Serialize)]
pub struct SageAppIdentity {
    id: String,
    origin_id: String,
    app_dir: String,
}

#[derive(Debug, Serialize)]
pub struct SageAppCommon {
    identity: SageAppIdentity,
    granted_permissions: SageGrantedPermissions,
    storage: SageAppStorage,
    origin_webview_storage_may_contain_secrets: bool,
    active_snapshot: SageAppSnapshot,
    wallet_scope: SageAppWalletScope,
}

impl SageAppCommon {
    pub(crate) fn new(
        identity: SageAppIdentity,
        granted_permissions: SageGrantedPermissions,
        storage: SageAppStorage,
        snapshot: SageAppSnapshot,
        wallet_scope: SageAppWalletScope,
    ) -> anyhow::Result<Self> {
        Self::build(
            identity,
            granted_permissions,
            storage,
            false,
            snapshot,
            wallet_scope,
        )
    }

    pub(crate) fn from_persisted_parts(
        identity: SageAppIdentity,
        granted_permissions: SageGrantedPermissions,
        storage: SageAppStorage,
        origin_webview_storage_may_contain_secrets: bool,
        snapshot: SageAppSnapshot,
        wallet_scope: SageAppWalletScope,
    ) -> anyhow::Result<Self> {
        Self::build(
            identity,
            granted_permissions,
            storage,
            origin_webview_storage_may_contain_secrets,
            snapshot,
            wallet_scope,
        )
    }

    pub(crate) fn apply_update(
        &mut self,
        pending: &UserSageAppPendingUpdate,
        granted_permissions: SageGrantedPermissions,
        snapshot: SageAppSnapshot,
    ) -> anyhow::Result<()> {
        let next = Self::build(
            self.identity.clone(),
            granted_permissions,
            self.storage.clone(),
            self.origin_webview_storage_may_contain_secrets,
            snapshot,
            self.wallet_scope.clone(),
        )?;

        if next.active_snapshot.manifest() != pending.manifest() {
            anyhow::bail!("update snapshot manifest does not match pending update manifest");
        }

        *self = next;
        Ok(())
    }

    pub(crate) fn update_permissions(
        &mut self,
        granted_permissions: &SageGrantedPermissions,
    ) -> anyhow::Result<()> {
        let requested = self.active_manifest().permissions();

        let required_network = requested.network().whitelist().required().cloned();

        let granted_network = granted_permissions.network().whitelist_iter().cloned();

        let mut whitelist_by_network = granted_permissions.network().whitelist_by_network().clone();

        for (network_id, whitelist) in requested.network().whitelist_by_network() {
            whitelist_by_network
                .entry(network_id.clone())
                .or_default()
                .extend(whitelist.required().cloned());
        }

        let granted_permissions = SageGrantedPermissions::new(
            requested,
            granted_permissions.capabilities().copied(),
            required_network.chain(granted_network),
            whitelist_by_network,
        )?;

        let next = Self::build(
            self.identity.clone(),
            granted_permissions,
            self.storage.clone(),
            self.origin_webview_storage_may_contain_secrets,
            self.active_snapshot.clone(),
            self.wallet_scope.clone(),
        )?;

        *self = next;
        Ok(())
    }

    pub(crate) fn replace_storage_and_origin(
        &mut self,
        storage: SageAppStorage,
        origin_id: impl Into<String>,
        origin_webview_storage_may_contain_secrets: bool,
    ) -> anyhow::Result<()> {
        let next = Self::build(
            SageAppIdentity::new(self.id().to_string(), origin_id, self.app_dir().to_string())?,
            self.granted_permissions.clone(),
            storage,
            origin_webview_storage_may_contain_secrets,
            self.active_snapshot.clone(),
            self.wallet_scope.clone(),
        )?;

        *self = next;
        Ok(())
    }

    pub(crate) fn mark_origin_webview_storage_may_contain_secrets(&mut self) -> anyhow::Result<()> {
        self.replace_origin_webview_storage_may_contain_secrets(true)
    }

    pub(crate) fn replace_origin_webview_storage_may_contain_secrets(
        &mut self,
        origin_webview_storage_may_contain_secrets: bool,
    ) -> anyhow::Result<()> {
        let next = Self::build(
            self.identity.clone(),
            self.granted_permissions.clone(),
            self.storage.clone(),
            origin_webview_storage_may_contain_secrets,
            self.active_snapshot.clone(),
            self.wallet_scope.clone(),
        )?;

        *self = next;
        Ok(())
    }

    pub(crate) fn origin_webview_storage_may_contain_secrets(&self) -> bool {
        self.origin_webview_storage_may_contain_secrets
    }

    pub(crate) fn clone_durable(&self) -> Self {
        Self {
            identity: self.identity.clone(),
            granted_permissions: self.granted_permissions.clone(),
            storage: self.storage.clone(),
            origin_webview_storage_may_contain_secrets: self
                .origin_webview_storage_may_contain_secrets,
            active_snapshot: self.active_snapshot.clone(),
            wallet_scope: self.wallet_scope.clone(),
        }
    }

    pub(crate) fn capability_flags(&self) -> CapabilityFlags {
        CapabilityFlags::from_capabilities(
            &self.granted_permissions.capabilities_vec(),
        )
    }

    pub(crate) fn has_secret_access(&self) -> bool {
        self.capability_flags().accesses_sensitive_secret()
    }

    pub(crate) fn has_external_access(&self) -> bool {
        self.capability_flags().externally_observable()
            || !self
                .granted_permissions
                .network()
                .all_whitelist_entries()
                .is_empty()
    }

    pub(crate) fn has_persistent_webview_storage(&self) -> bool {
        self.granted_permissions()
            .capabilities()
            .any(|cap| *cap == UserBridgeCapability::StoragePersistentWebview)
    }

    fn build(
        identity: SageAppIdentity,
        granted_permissions: SageGrantedPermissions,
        storage: SageAppStorage,
        origin_webview_storage_may_contain_secrets: bool,
        snapshot: SageAppSnapshot,
        wallet_scope: SageAppWalletScope,
    ) -> anyhow::Result<Self> {
        let manifest = snapshot.manifest();

        let granted_permissions = SageGrantedPermissions::from_requested_and_granted(
            manifest.permissions(),
            granted_permissions,
        )?;

        let common = Self {
            identity,
            granted_permissions,
            storage,
            origin_webview_storage_may_contain_secrets,
            active_snapshot: snapshot,
            wallet_scope,
        };

        if common.has_external_access() && common.has_secret_access() {
            anyhow::bail!(
                "app permissions cannot include both external access and sensitive secret access"
            );
        }
        if common.has_external_access() && common.origin_webview_storage_may_contain_secrets {
            anyhow::bail!(
                "app permissions cannot include external access while origin webview storage may contain secrets"
            );
        }

        validate_snapshot_entry_and_icon_exist(
            &common.active_snapshot,
            common.entry_file(),
            common.icon_file(),
            "app",
        )?;

        Ok(common)
    }

    pub fn is_wallet_in_scope(&self, fingerprint: u32) -> bool {
        match self.wallet_scope() {
            SageAppWalletScope::AllWallets => true,
            SageAppWalletScope::SelectedWallets { fingerprints } => {
                fingerprints.contains(&fingerprint)
            }
        }
    }

    pub fn identity(&self) -> &SageAppIdentity {
        &self.identity
    }
    pub fn id(&self) -> &str {
        &self.identity.id
    }

    pub fn origin_id(&self) -> &str {
        self.identity.origin_id()
    }

    pub fn name(&self) -> &str {
        self.active_manifest().name()
    }

    pub fn version(&self) -> &str {
        self.active_manifest().version()
    }

    pub fn app_dir(&self) -> &str {
        &self.identity.app_dir
    }

    pub fn app_path(&self) -> PathBuf {
        PathBuf::from(&self.identity.app_dir)
    }

    pub fn entry_file(&self) -> &str {
        self.active_manifest().entry()
    }

    pub fn icon_file(&self) -> Option<&str> {
        self.active_manifest().icon()
    }

    pub fn requested_permissions(&self) -> &SageRequestedPermissions {
        self.active_manifest().permissions()
    }

    pub fn granted_permissions(&self) -> &SageGrantedPermissions {
        &self.granted_permissions
    }

    pub fn storage(&self) -> &SageAppStorage {
        &self.storage
    }

    pub fn wallet_scope(&self) -> &SageAppWalletScope {
        &self.wallet_scope
    }

    pub(crate) fn update_wallet_scope(&mut self, wallet_scope: SageAppWalletScope) {
        self.wallet_scope = wallet_scope;
    }

    pub fn active_snapshot(&self) -> &SageAppSnapshot {
        &self.active_snapshot
    }

    pub fn active_manifest(&self) -> &SageAppPackageManifest {
        self.active_snapshot.manifest()
    }

    pub fn entry_path(&self) -> PathBuf {
        self.active_snapshot
            .file_path(self.active_manifest().entry())
    }

    pub fn icon_path(&self) -> Option<PathBuf> {
        self.active_manifest()
            .icon()
            .as_ref()
            .map(|icon_file| self.active_snapshot.file_path(icon_file))
    }

    pub fn is_sandbox_test(&self) -> bool {
        self.id().starts_with(SANDBOX_TEST_ID_PREFIX)
    }
}

impl SageAppIdentity {
    pub fn new(
        id: impl Into<String>,
        origin_id: impl Into<String>,
        app_dir: impl Into<String>,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            id: normalized_non_empty_string(id, "app id")?,
            origin_id: normalized_non_empty_string(origin_id, "app origin id")?,
            app_dir: normalized_non_empty_string(app_dir, "app directory")?,
        })
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn origin_id(&self) -> &str {
        &self.origin_id
    }

    pub fn app_dir(&self) -> &str {
        &self.app_dir
    }
}

#[derive(Debug, Deserialize)]
struct SageAppIdentityRaw {
    id: String,
    origin_id: String,
    app_dir: String,
}

impl<'de> Deserialize<'de> for SageAppIdentity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = SageAppIdentityRaw::deserialize(deserializer)?;

        SageAppIdentity::new(raw.id, raw.origin_id, raw.app_dir).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct SageAppCommonRaw {
    identity: SageAppIdentity,
    granted_permissions: SageGrantedPermissions,
    storage: SageAppStorage,
    origin_webview_storage_may_contain_secrets: bool,
    active_snapshot: SageAppSnapshot,
    wallet_scope: SageAppWalletScope,
}

impl TryFrom<SageAppCommonRaw> for SageAppCommon {
    type Error = anyhow::Error;

    fn try_from(raw: SageAppCommonRaw) -> anyhow::Result<Self> {
        SageAppCommon::build(
            raw.identity,
            raw.granted_permissions,
            raw.storage,
            raw.origin_webview_storage_may_contain_secrets,
            raw.active_snapshot,
            raw.wallet_scope,
        )
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::fs;

    use tempfile::{TempDir, tempdir};

    use super::*;
    use crate::{SageAppManifestFile, SageAppManifestSageVersion, SageAppManifestVersion, SageAppPackageManifest, SageAppPackageManifestParts, SageNetworkWhitelistEntry, SageRequestedCapabilities, SageRequestedNetworkPermissions, SageRequestedPermissions, UserBridgeCapability};

    fn entry(scheme: &str, host: &str) -> SageNetworkWhitelistEntry {
        SageNetworkWhitelistEntry::new(scheme, host).unwrap()
    }

    fn manifest(permissions: SageRequestedPermissions) -> SageAppPackageManifest {
        SageAppPackageManifest::try_from(SageAppPackageManifestParts {
            manifest_version: SageAppManifestVersion(0),
            name: "test app".to_string(),
            icon: None,
            sage_version: SageAppManifestSageVersion {
                min: "0.0.0".to_string(),
                tested_max: None,
            },
            version: "1.0.0".to_string(),
            permissions,
            files: vec![SageAppManifestFile::new("index.html", "a".repeat(64), 1).unwrap()],
            entry: Some("index.html".to_string()),
            author: None,
            donation: None,
        })
        .unwrap()
    }

    fn tainted_app() -> (SageAppCommon, SageNetworkWhitelistEntry, TempDir) {
        let optional_network = entry("https", "optional.example.com");
        let requested = SageRequestedPermissions::new(
            SageRequestedNetworkPermissions::new([], [optional_network.clone()], []).unwrap(),
            SageRequestedCapabilities::new([], [UserBridgeCapability::StoragePersistentWebview]),
        )
        .unwrap();
        let granted = SageGrantedPermissions::new(
            &requested,
            [UserBridgeCapability::StoragePersistentWebview],
            [],
            BTreeMap::new(),
        )
        .unwrap();

        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();
        fs::write(dir_path.join("index.html"), "x").unwrap();
        let snapshot =
            SageAppSnapshot::new("hash", dir_path.to_string_lossy(), manifest(requested)).unwrap();

        let app = SageAppCommon::from_persisted_parts(
            SageAppIdentity::new("app-id", "origin-id", dir_path.to_string_lossy()).unwrap(),
            granted,
            SageAppStorage::Unmanaged,
            true,
            snapshot,
            SageAppWalletScope::AllWallets,
        )
        .unwrap();

        (app, optional_network, dir)
    }

    #[test]
    fn update_permissions_rejects_external_access_when_origin_storage_may_contain_secrets() {
        let (mut app, optional_network, _dir) = tainted_app();

        let granted_with_network = app
            .granted_permissions()
            .with_network_whitelist_entry_added(app.requested_permissions(), optional_network)
            .unwrap();

        let err = app.update_permissions(&granted_with_network).unwrap_err();

        assert!(
            err.to_string()
                .contains("external access while origin webview storage may contain secrets"),
            "unexpected error: {err}"
        );
    }
}

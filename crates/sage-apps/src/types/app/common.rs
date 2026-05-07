use std::path::PathBuf;
use serde::{Deserialize, Deserializer, Serialize};
use crate::sandbox::SANDBOX_TEST_ID_PREFIX;
use crate::types::app::flags::SageAppFlags;
use crate::types::app::preview::UserSageAppPendingUpdate;
use crate::types::app::SageAppWalletScope;
use crate::types::app::snapshot::SageAppSnapshot;
use crate::types::invariants::{
    resolve_app_capability_flags, validate_snapshot_entry_and_icon_exist,
};
use crate::types::normalizers::normalized_non_empty_string;
use crate::types::permissions::{SageGrantedPermissions, SageRequestedPermissions};
use crate::types::storage::InstalledSageAppStorage;

#[derive(Debug, Clone, Serialize)]
pub struct SageAppIdentity {
    id: String,
    origin_id: String,
    app_dir: String,
}

#[derive(Debug)]
pub struct SageAppCommon {
    identity: SageAppIdentity,
    granted_permissions: SageGrantedPermissions,
    flags: SageAppFlags,
    storage: InstalledSageAppStorage,
    active_snapshot: SageAppSnapshot,
    wallet_scope: SageAppWalletScope,
}

impl SageAppCommon {
    pub fn new(
        identity: SageAppIdentity,
        granted_permissions: SageGrantedPermissions,
        storage: InstalledSageAppStorage,
        snapshot: SageAppSnapshot,
        wallet_scope: SageAppWalletScope,
    ) -> anyhow::Result<Self> {
        Self::build(identity, granted_permissions, storage, snapshot, wallet_scope, None)
    }

    pub fn clone_for_rollback(&self) -> Self {
        Self {
            identity: self.identity.clone(),
            granted_permissions: self.granted_permissions.clone(),
            storage: self.storage.clone(),
            flags: self.flags,
            active_snapshot: self.active_snapshot.clone(),
            wallet_scope: self.wallet_scope.clone(),
        }
    }

    pub fn apply_update(
        &mut self,
        pending: &UserSageAppPendingUpdate,
        granted_permissions: SageGrantedPermissions,
        snapshot: SageAppSnapshot,
    ) -> anyhow::Result<()> {
        let next = Self::build(
            self.identity.clone(),
            granted_permissions,
            self.storage.clone(),
            snapshot,
            self.wallet_scope.clone(),
            Some(&self.flags),
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
        let required_network = self
            .active_manifest()
            .permissions()
            .network()
            .whitelist()
            .required()
            .cloned();

        let granted_network = granted_permissions.network().whitelist_iter().cloned();

        let granted_permissions = SageGrantedPermissions::new(
            self.active_manifest().permissions(),
            granted_permissions.capabilities().copied(),
            required_network.chain(granted_network),
        )?;

        let next = Self::build(
            self.identity.clone(),
            granted_permissions,
            self.storage.clone(),
            self.active_snapshot.clone(),
            self.wallet_scope.clone(),
            Some(&self.flags),
        )?;

        *self = next;
        Ok(())
    }

    pub(crate) fn mark_storage_may_contain_secrets(&mut self) {
        self.flags.mark_storage_may_contain_secrets();
    }

    pub(crate) fn clear_storage_may_contain_secrets(&mut self) {
        self.flags.clear_storage_may_contain_secrets();
    }

    fn build(
        identity: SageAppIdentity,
        granted_permissions: SageGrantedPermissions,
        storage: InstalledSageAppStorage,
        snapshot: SageAppSnapshot,
        wallet_scope: SageAppWalletScope,
        previous_flags: Option<&SageAppFlags>,
    ) -> anyhow::Result<Self> {
        let manifest = snapshot.manifest();

        let granted_permissions = SageGrantedPermissions::from_requested_and_granted(
            manifest.permissions(),
            granted_permissions,
        )?;

        let capability_flags =
            resolve_app_capability_flags(manifest, &granted_permissions, previous_flags)?;

        let common = Self {
            identity,
            granted_permissions,
            flags: capability_flags,
            storage,
            active_snapshot: snapshot,
            wallet_scope,
        };

        validate_snapshot_entry_and_icon_exist(
            &common.active_snapshot,
            common.entry_file(),
            common.icon_file(),
            "app",
        )?;

        Ok(common)
    }

    pub fn identity(&self) -> &SageAppIdentity { &self.identity }
    pub fn id(&self) -> &str {
        &self.identity.id
    }

    pub fn origin_id(&self) -> &str {
        &self.identity.origin_id
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

    pub fn flags(&self) -> &SageAppFlags {
        &self.flags
    }

    pub fn storage(&self) -> &InstalledSageAppStorage {
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

    pub fn active_manifest(&self) -> &crate::types::manifest::SageAppPackageManifest {
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

        SageAppIdentity::new(
            raw.id,
            raw.origin_id,
            raw.app_dir,
        )
            .map_err(serde::de::Error::custom)
    }
}

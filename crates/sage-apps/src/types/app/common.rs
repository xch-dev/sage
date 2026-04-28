use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::lifecycle::{manifest_entry_file, manifest_icon_file};
use crate::types::app::flags::SageAppFlags;
use crate::types::app::preview::UserSageAppPendingUpdate;
use crate::types::app::snapshot::SageAppSnapshot;
use crate::types::normalizers::normalized_non_empty_string;
use crate::types::permissions::{SageGrantedPermissions, SageRequestedPermissions};
use crate::types::storage::InstalledSageAppStorage;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SageAppCommon {
    id: String,
    origin_id: String,
    name: String,
    version: String,
    app_dir: String,
    entry_file: String,
    icon_file: Option<String>,
    requested_permissions: SageRequestedPermissions,
    granted_permissions: SageGrantedPermissions,
    capability_flags: SageAppFlags,
    storage: InstalledSageAppStorage,
    active_snapshot: SageAppSnapshot,
}

impl SageAppCommon {
    pub fn new(
        id: impl Into<String>,
        origin_id: impl Into<String>,
        app_dir: impl Into<String>,
        granted_permissions: SageGrantedPermissions,
        storage: InstalledSageAppStorage,
        snapshot: SageAppSnapshot,
    ) -> anyhow::Result<Self> {
        Self::build(
            id.into(),
            origin_id.into(),
            app_dir.into(),
            granted_permissions,
            storage,
            snapshot,
            None,
        )
    }

    pub fn apply_update(
        &mut self,
        pending: &UserSageAppPendingUpdate,
        granted_permissions: SageGrantedPermissions,
        snapshot: SageAppSnapshot,
    ) -> anyhow::Result<()> {
        let next = Self::build(
            self.id.clone(),
            self.origin_id.clone(),
            self.app_dir.clone(),
            granted_permissions,
            self.storage.clone(),
            snapshot,
            Some(&self.capability_flags),
        )?;

        if next.active_snapshot.manifest() != pending.manifest() {
            anyhow::bail!("update snapshot manifest does not match pending update manifest");
        }

        *self = next;
        Ok(())
    }

    pub fn update_permissions(
        &mut self,
        granted_permissions: &SageGrantedPermissions,
    ) -> anyhow::Result<()> {
        let required_network = self
            .requested_permissions
            .network()
            .whitelist()
            .required()
            .cloned();

        let granted_network = granted_permissions.network().whitelist().cloned();

        let granted_permissions = SageGrantedPermissions::new(
            &self.requested_permissions,
            granted_permissions.capabilities().copied(),
            required_network.chain(granted_network),
        )?;

        let next = Self::build(
            self.id.clone(),
            self.origin_id.clone(),
            self.app_dir.clone(),
            granted_permissions,
            self.storage.clone(),
            self.active_snapshot.clone(),
            Some(&self.capability_flags),
        )?;

        *self = next;
        Ok(())
    }

    pub fn mark_storage_may_contain_secrets(&mut self) {
        self.capability_flags.mark_storage_may_contain_secrets();
    }

    fn build(
        id: String,
        origin_id: String,
        app_dir: String,
        granted_permissions: SageGrantedPermissions,
        storage: InstalledSageAppStorage,
        snapshot: SageAppSnapshot,
        previous_flags: Option<&SageAppFlags>,
    ) -> anyhow::Result<Self> {
        let id = normalized_non_empty_string(id, "app id")?;
        let origin_id = normalized_non_empty_string(origin_id, "app origin id")?;
        let app_dir = normalized_non_empty_string(app_dir, "app directory")?;

        let manifest = snapshot.manifest();

        let granted_permissions =
            SageGrantedPermissions::from_requested_and_granted(
                manifest.permissions(),
                granted_permissions,
            )?;

        let effective_capabilities = manifest
            .permissions()
            .capabilities()
            .resolve_effective_grants(granted_permissions.capabilities().copied())?;

        let capability_flags =
            SageAppFlags::from_granted_capabilities(&effective_capabilities, previous_flags)?;
        Self::validate_app_flags_policy(capability_flags)?;

        let common = Self {
            id,
            origin_id,
            name: manifest.name().to_string(),
            version: manifest.version().to_string(),
            app_dir,
            entry_file: manifest_entry_file(manifest).to_string(),
            icon_file: manifest_icon_file(manifest).map(str::to_string),
            requested_permissions: manifest.permissions().clone(),
            granted_permissions,
            capability_flags,
            storage,
            active_snapshot: snapshot,
        };

        common.validate_files_exist_internal("app")?;

        Ok(common)
    }

    fn validate_app_flags_policy(flags: SageAppFlags) -> anyhow::Result<()> {
        if flags.has_external_access() && flags.has_secret_access() {
            anyhow::bail!(
                "app permissions cannot include both external access and sensitive secret access"
            );
        }

        if flags.has_external_access() && flags.storage_may_contain_secrets() {
            anyhow::bail!(
                "app permissions cannot include external access while storage may contain secrets"
            );
        }

        Ok(())
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn origin_id(&self) -> &str {
        &self.origin_id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn app_dir(&self) -> &str {
        &self.app_dir
    }

    pub fn entry_file(&self) -> &str {
        &self.entry_file
    }

    pub fn icon_file(&self) -> Option<&str> {
        self.icon_file.as_deref()
    }

    pub fn requested_permissions(&self) -> &SageRequestedPermissions {
        &self.requested_permissions
    }

    pub fn granted_permissions(&self) -> &SageGrantedPermissions {
        &self.granted_permissions
    }

    pub fn capability_flags(&self) -> &SageAppFlags {
        &self.capability_flags
    }

    pub fn storage(&self) -> &InstalledSageAppStorage {
        &self.storage
    }

    pub fn active_snapshot(&self) -> &SageAppSnapshot {
        &self.active_snapshot
    }

    pub fn entry_path(&self) -> PathBuf {
        Path::new(&self.app_dir).join(&self.entry_file)
    }

    pub fn icon_path(&self) -> Option<PathBuf> {
        self.icon_file
            .as_ref()
            .map(|icon_file| Path::new(&self.app_dir).join(icon_file))
    }

    fn validate_files_exist_internal(&self, label: &str) -> anyhow::Result<()> {
        let entry_file = self.entry_path();

        if !entry_file.is_file() {
            anyhow::bail!("{label} entry file does not exist: {}", entry_file.display());
        }

        if let Some(icon_file) = self.icon_path()
            && !icon_file.is_file()
        {
            anyhow::bail!("{label} icon file does not exist: {}", icon_file.display());
        }

        Ok(())
    }
}

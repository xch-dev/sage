use serde::{Deserialize, Serialize};
use specta::Type;
use uuid::Uuid;
use crate::lifecycle::flags::get_app_flags;
use crate::lifecycle::{manifest_entry_file, manifest_icon_file};
use crate::sandbox::SANDBOX_TEST_ID_PREFIX;
use crate::types::manifest::SageAppPackageManifest;
use crate::types::permissions::{SageGrantedPermissions, SageGrantedSystemPermissions, SageRequestedPermissions};
use crate::types::storage::InstalledSageAppStorage;
use crate::utils::unix_timestamp_ms;

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, Default)]
#[serde(rename_all = "camelCase")]
pub struct SageAppFlags {
    pub has_secret_access: bool,
    pub has_external_access: bool,
    pub storage_may_contain_secrets: bool,
    pub isolated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SageAppSnapshot {
    pub manifest_hash: String,
    pub snapshot_dir: String,
    pub total_bytes: u64,
    pub manifest: SageAppPackageManifest,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct UserSageAppPendingUpdate {
    pub app_url: String,
    pub manifest_url: String,
    pub manifest_hash: String,
    pub manifest: SageAppPackageManifest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum UserSageAppSource {
    Zip,
    Url {
        app_url: String,
        manifest_url: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SageAppCommon {
    pub id: String,
    pub origin_id: String,
    pub name: String,
    pub version: String,
    pub app_dir: String,
    pub entry_file: String,
    pub icon_file: String,
    pub requested_permissions: SageRequestedPermissions,
    pub granted_permissions: SageGrantedPermissions,
    pub capability_flags: SageAppFlags,
    pub storage: InstalledSageAppStorage,
    pub active_snapshot: SageAppSnapshot,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq)]
pub enum SystemAppPresentation {
    Taskbar,
    Modal,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct UserSageApp {
    pub common: SageAppCommon,
    pub source: UserSageAppSource,
    pub pending_update: Option<UserSageAppPendingUpdate>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SystemSageApp {
    pub common: SageAppCommon,
    pub system_granted_permissions: SageGrantedSystemPermissions,
    pub presentation: SystemAppPresentation,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[allow(clippy::large_enum_variant)]
pub enum SageApp {
    System(SystemSageApp),
    User(UserSageApp),
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SageAppId {
    pub id: String,
    pub origin_id: String,
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CorruptedInstalledSageApp {
    pub id: String,
    pub app_dir: String,
    pub error: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
pub struct SageAppAuthor {
    pub name: String,
    pub avatar: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
pub struct SageAppDonation {
    pub address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SageAppUrlPreview {
    pub app_url: String,
    pub manifest_url: String,
    pub manifest_hash: String,
    pub manifest: SageAppPackageManifest,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[allow(clippy::large_enum_variant)]
pub enum ListedSageApp {
    User(UserSageApp),
    System(SystemSageApp),
    Corrupted(CorruptedInstalledSageApp),
}

impl SageApp {
    pub fn common(&self) -> &SageAppCommon {
        match self {
            Self::System(app) => &app.common,
            Self::User(app) => &app.common,
        }
    }

    pub fn id(&self) -> &str {
        &self.common().id
    }

    pub fn origin_id(&self) -> &str {
        &self.common().origin_id
    }

    pub fn name(&self) -> &str {
        &self.common().name
    }

    pub fn version(&self) -> &str {
        &self.common().version
    }

    pub fn app_dir(&self) -> &str {
        &self.common().app_dir
    }

    pub fn entry_file(&self) -> &str {
        &self.common().entry_file
    }

    pub fn icon_file(&self) -> &str {
        &self.common().icon_file
    }

    pub fn requested_permissions(&self) -> &SageRequestedPermissions {
        &self.common().requested_permissions
    }

    pub fn granted_permissions(&self) -> &SageGrantedPermissions {
        &self.common().granted_permissions
    }

    pub fn system_granted_permissions(&self) -> Option<&SageGrantedSystemPermissions> {
        match self {
            Self::System(app) => Some(&app.system_granted_permissions),
            Self::User(_) => None,
        }
    }

    pub fn capability_flags(&self) -> &SageAppFlags {
        &self.common().capability_flags
    }

    pub fn storage(&self) -> &InstalledSageAppStorage {
        &self.common().storage
    }

    pub fn active_snapshot(&self) -> &SageAppSnapshot {
        &self.common().active_snapshot
    }

    pub fn as_user(&self) -> Option<&UserSageApp> {
        match self {
            Self::User(app) => Some(app),
            Self::System(_) => None,
        }
    }

    pub fn as_user_mut(&mut self) -> Option<&mut UserSageApp> {
        match self {
            Self::User(app) => Some(app),
            Self::System(_) => None,
        }
    }

    pub fn as_system(&self) -> Option<&SystemSageApp> {
        match self {
            Self::System(app) => Some(app),
            Self::User(_) => None,
        }
    }

    pub fn as_system_mut(&mut self) -> Option<&mut SystemSageApp> {
        match self {
            Self::System(app) => Some(app),
            Self::User(_) => None,
        }
    }

    pub fn into_user(self) -> Option<UserSageApp> {
        match self {
            Self::User(app) => Some(app),
            Self::System(_) => None,
        }
    }

    pub fn into_system(self) -> Option<SystemSageApp> {
        match self {
            Self::System(app) => Some(app),
            Self::User(_) => None,
        }
    }

    pub fn is_sandbox_test(&self) -> bool {
        self.id().starts_with(SANDBOX_TEST_ID_PREFIX)
    }
}

impl UserSageApp {
    pub fn into_sage_app(self) -> SageApp {
        SageApp::User(self)
    }
}

impl SystemSageApp {
    pub fn into_sage_app(self) -> SageApp {
        SageApp::System(self)
    }
}

impl SageAppCommon {
    pub fn new(
        id: String,
        origin_id: String,
        app_dir: String,
        manifest: &SageAppPackageManifest,
        granted_permissions: SageGrantedPermissions,
        storage: InstalledSageAppStorage,
        snapshot: SageAppSnapshot,
    ) -> anyhow::Result<Self> {
        let granted_permissions = SageGrantedPermissions::from_requested_and_granted(
            manifest.permissions(),
            granted_permissions,
        )?;

        let effective_capabilities = manifest
            .permissions()
            .capabilities
            .resolve_effective_grants(granted_permissions.capabilities().copied())?;

        let capability_flags = get_app_flags(&effective_capabilities, None)?;
        Self::validate_app_flags_policy(capability_flags)?;

        Ok(Self {
            id,
            origin_id,
            name: manifest.name().to_string(),
            version: manifest.version().to_string(),
            app_dir,
            entry_file: manifest_entry_file(manifest).to_string(),
            icon_file: manifest_icon_file(manifest).to_string(),
            requested_permissions: manifest.permissions().clone(),
            granted_permissions,
            capability_flags,
            storage,
            active_snapshot: snapshot,
        })
    }

    pub fn apply_update(
        &mut self,
        pending: &UserSageAppPendingUpdate,
        granted_permissions: SageGrantedPermissions,
        snapshot: SageAppSnapshot,
    ) -> anyhow::Result<()> {
        let granted_permissions = SageGrantedPermissions::from_requested_and_granted(
            pending.manifest.permissions(),
            granted_permissions,
        )?;

        let effective_capabilities = pending
            .manifest
            .permissions()
            .capabilities
            .resolve_effective_grants(granted_permissions.capabilities().copied())?;

        let capability_flags =
            get_app_flags(&effective_capabilities, Some(&self.capability_flags))?;

        Self::validate_app_flags_policy(capability_flags)?;

        self.name.clone_from(&pending.manifest.name().to_string());
        self.version.clone_from(&pending.manifest.version().to_string());
        self.requested_permissions
            .clone_from(pending.manifest.permissions());
        self.granted_permissions = granted_permissions;
        self.capability_flags = capability_flags;
        self.entry_file = manifest_entry_file(&snapshot.manifest).to_string();
        self.icon_file = manifest_icon_file(&snapshot.manifest).to_string();
        self.active_snapshot = snapshot;

        Ok(())
    }

    pub fn update_permissions(
        &mut self,
        granted_permissions: &SageGrantedPermissions,
    ) -> anyhow::Result<()> {
        let required_network = self
            .requested_permissions
            .network
            .whitelist()
            .required()
            .cloned();

        let granted_network = granted_permissions.network().whitelist().cloned();

        let granted_permissions = SageGrantedPermissions::new(
            &self.requested_permissions,
            granted_permissions.capabilities().copied(),
            required_network.chain(granted_network),
        )?;

        let effective_capabilities = self
            .requested_permissions
            .capabilities
            .resolve_effective_grants(granted_permissions.capabilities().copied())?;

        let capability_flags =
            get_app_flags(&effective_capabilities, Some(&self.capability_flags))?;

        Self::validate_app_flags_policy(capability_flags)?;

        self.capability_flags = capability_flags;
        self.granted_permissions = granted_permissions;

        Ok(())
    }

    fn validate_app_flags_policy(flags: SageAppFlags) -> anyhow::Result<()> {
        if flags.has_external_access && flags.has_secret_access {
            anyhow::bail!(
                "app permissions cannot include both external access and sensitive secret access"
            );
        }

        if flags.has_external_access && flags.storage_may_contain_secrets {
            anyhow::bail!(
                "app permissions cannot include external access while storage may contain secrets"
            );
        }

        Ok(())
    }
}

impl UserSageApp {
    pub fn new_installed(
        common: SageAppCommon,
        source: UserSageAppSource,
    ) -> Self {
        Self {
            common,
            source,
            pending_update: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RetiredAppOriginEntry {
    id: String,
    app_id: String,
    app_name: String,
    origin_id: String,
    created_at_ms: i64,
    storage_may_contain_secrets: bool,
    cleanup_pending: bool,
}

impl RetiredAppOriginEntry {
    pub fn new(app: &UserSageApp, cleanup_pending: bool) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            app_id: app.common.id.clone(),
            app_name: app.common.name.clone(),
            origin_id: app.common.origin_id.clone(),
            created_at_ms: unix_timestamp_ms(),
            storage_may_contain_secrets: app.common.capability_flags.storage_may_contain_secrets,
            cleanup_pending,
        }
    }

    pub fn refresh_from_app(&mut self, app: &UserSageApp, cleanup_pending: bool) {
        self.app_id.clone_from(&app.common.id);
        self.app_name.clone_from(&app.common.name);
        self.cleanup_pending = cleanup_pending;
        self.storage_may_contain_secrets =
            app.common.capability_flags.storage_may_contain_secrets;
    }


    pub fn matches_app_origin(&self, app_id: &str, origin_id: &str) -> bool {
        self.app_id == app_id && self.origin_id == origin_id
    }

    pub fn clear_pending_cleanup(&mut self) -> bool {
        if !self.cleanup_pending {
            return false;
        }

        self.cleanup_pending = false;
        true
    }

    pub fn app_id(&self) -> &str {
        &self.app_id
    }

    pub fn origin_id(&self) -> &str {
        &self.origin_id
    }

    pub fn cleanup_pending(&self) -> bool {
        self.cleanup_pending
    }

    pub fn storage_may_contain_secrets(&self) -> bool {
        self.storage_may_contain_secrets
    }
}

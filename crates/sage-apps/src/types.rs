use serde::{Deserialize, Deserializer, Serialize};
use specta::Type;

use crate::bridge::capabilities::{SystemBridgeCapability, UserBridgeCapability};
use crate::lifecycle::parse_network_permission_target;
use crate::sandbox::SANDBOX_TEST_ID_PREFIX;

#[derive(
    Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq, PartialOrd, Ord,
)]
pub struct SageNetworkPermissionTarget {
    pub scheme: String,
    pub host: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, Default, PartialEq, Eq)]
pub struct SageRequestedNetworkWhitelist {
    pub required: Vec<SageNetworkPermissionTarget>,
    pub optional: Vec<SageNetworkPermissionTarget>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, Default, PartialEq, Eq)]
pub struct SageRequestedNetworkPermissions {
    pub whitelist: SageRequestedNetworkWhitelist,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, Default, PartialEq, Eq)]
pub struct SageRequestedCapabilities {
    pub required: Vec<UserBridgeCapability>,
    pub optional: Vec<UserBridgeCapability>,
}

#[derive(Debug, Clone, Serialize, Type, Default, PartialEq, Eq)]
pub struct SageRequestedPermissions {
    pub network: SageRequestedNetworkPermissions,
    pub capabilities: SageRequestedCapabilities,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, Default, PartialEq, Eq)]
pub struct SageGrantedNetworkPermissions {
    pub whitelist: Vec<SageNetworkPermissionTarget>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, Default, PartialEq, Eq)]
pub struct SageGrantedPermissions {
    pub capabilities: Vec<UserBridgeCapability>,
    pub network: SageGrantedNetworkPermissions,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SageGrantedSystemPermissions {
    pub capabilities: Vec<SystemBridgeCapability>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum InstalledSageAppStorage {
    AppleDataStore { identifier_hex: String },
    WindowsProfile { directory_name: String },
    Unmanaged,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum PendingStorageCleanupTarget {
    AppleDataStore { identifier_hex: String },
    WindowsProfile { directory_name: String },
    Unmanaged,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PendingStorageCleanupEntry {
    pub id: String,
    pub app_id: String,
    pub app_name: String,
    pub target: PendingStorageCleanupTarget,
    pub created_at_ms: i64,
    pub last_attempt_at_ms: Option<i64>,
    pub attempt_count: u32,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RetiredAppOriginEntry {
    pub id: String,
    pub app_id: String,
    pub app_name: String,
    pub origin_id: String,
    pub created_at_ms: i64,
    pub storage_may_contain_secrets: bool,
    pub cleanup_pending: bool,
}

#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
pub struct SageAppPackageManifest {
    pub name: String,
    pub version: String,
    pub permissions: SageRequestedPermissions,
    pub files: Vec<SageAppManifestFile>,
    pub entry: Option<String>,
    pub icon: Option<String>,
    pub author: Option<SageAppAuthor>,
    pub donation: Option<SageAppDonation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SageAppUrlPreview {
    pub app_url: String,
    pub manifest_url: String,
    pub manifest_hash: String,
    pub manifest: SageAppPackageManifest,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
pub struct SageAppManifestFile {
    pub path: String,
    pub sha256: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SageAppSnapshot {
    pub manifest_hash: String,
    pub snapshot_dir: String,
    pub total_bytes: u64,
    pub manifest: SageAppPackageManifest,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, Default)]
#[serde(rename_all = "camelCase")]
pub struct SageAppFlags {
    pub has_secret_access: bool,
    pub has_external_access: bool,
    pub storage_may_contain_secrets: bool,
    pub isolated: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SageAppCapabilityFlagsView {
    pub externally_observable: bool,
    pub accesses_sensitive_secret: bool,
    pub requestable_by_app: bool,
    pub user_grantable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SageAppCapabilityDefinitionView {
    pub key: String,
    pub label: String,
    pub description: String,
    pub flags: SageAppCapabilityFlagsView,
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
pub enum SageApp {
    System(SystemSageApp),
    User(UserSageApp),
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

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ListedSageApp {
    User(UserSageApp),
    System(SystemSageApp),
    Corrupted(CorruptedInstalledSageApp),
}

#[derive(Debug, Deserialize, Default)]
struct RawStringListBucket {
    #[serde(default)]
    required: Vec<String>,

    #[serde(default)]
    optional: Vec<String>,
}

#[derive(Debug, Deserialize, Default)]
struct RawRequestedNetworkPermissions {
    #[serde(default)]
    whitelist: RawStringListBucket,
}

#[derive(Debug, Deserialize, Default)]
struct RawRequestedPermissions {
    #[serde(default)]
    network: RawRequestedNetworkPermissions,

    #[serde(default)]
    capabilities: Option<SageRequestedCapabilities>,
}

#[derive(Debug, Deserialize, Default)]
struct RawSageAppPackageManifest {
    name: String,
    version: String,

    #[serde(default)]
    permissions: Option<SageRequestedPermissions>,

    #[serde(default)]
    files: Vec<SageAppManifestFile>,

    #[serde(default)]
    entry: Option<String>,

    #[serde(default)]
    icon: Option<String>,

    #[serde(default)]
    author: Option<SageAppAuthor>,

    #[serde(default)]
    donation: Option<SageAppDonation>,
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

impl<'de> Deserialize<'de> for SageRequestedPermissions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = <RawRequestedPermissions as Deserialize>::deserialize(deserializer)?;

        let required_network = raw
            .network
            .whitelist
            .required
            .into_iter()
            .map(|value| parse_network_permission_target(&value).map_err(serde::de::Error::custom))
            .collect::<Result<Vec<_>, _>>()?;

        let optional_network = raw
            .network
            .whitelist
            .optional
            .into_iter()
            .map(|value| parse_network_permission_target(&value).map_err(serde::de::Error::custom))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(SageRequestedPermissions {
            network: SageRequestedNetworkPermissions {
                whitelist: SageRequestedNetworkWhitelist {
                    required: required_network,
                    optional: optional_network,
                },
            },
            capabilities: raw.capabilities.unwrap_or_default(),
        })
    }
}

impl<'de> Deserialize<'de> for SageAppPackageManifest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = <RawSageAppPackageManifest as Deserialize>::deserialize(deserializer)?;

        Ok(SageAppPackageManifest {
            name: raw.name,
            version: raw.version,
            permissions: raw.permissions.unwrap_or_default(),
            files: raw.files,
            entry: raw.entry,
            icon: raw.icon,
            author: raw.author,
            donation: raw.donation,
        })
    }
}

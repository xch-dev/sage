use anyhow::anyhow;
use serde::{Deserialize, Deserializer, Serialize};
use specta::Type;
use std::collections::BTreeSet;
use uuid::Uuid;
use crate::bridge::capabilities::{
    SharedCapabilitiesExt, SystemBridgeCapability, UserBridgeCapability,
};
use crate::lifecycle::flags::get_app_flags;
use crate::lifecycle::{
    MAX_APP_FILE_COUNT, MAX_APP_TOTAL_SIZE_BYTES, manifest_entry_file, manifest_icon_file,
    validate_manifest_file_path, validate_sha256_hex,
};
use crate::permissions::{CapabilityFlags, get_user_capability_definition};
use crate::sandbox::SANDBOX_TEST_ID_PREFIX;
use crate::utils::unix_timestamp_ms;

#[derive(Debug, Clone, Type, PartialEq, Eq, PartialOrd, Ord)]
pub struct SageNetworkWhitelistEntry {
    scheme: String,
    host: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, Default, PartialEq, Eq)]
pub struct SageRequestedNetworkWhitelist {
    required: BTreeSet<SageNetworkWhitelistEntry>,
    optional: BTreeSet<SageNetworkWhitelistEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, Default, PartialEq, Eq)]
pub struct SageRequestedNetworkPermissions {
    pub whitelist: SageRequestedNetworkWhitelist,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, Default, PartialEq, Eq)]
pub struct SageRequestedCapabilities {
    required: BTreeSet<UserBridgeCapability>,
    optional: BTreeSet<UserBridgeCapability>,
}

#[derive(Debug, Clone, Serialize, Type, Default, PartialEq, Eq)]
pub struct SageRequestedPermissions {
    pub network: SageRequestedNetworkPermissions,
    pub capabilities: SageRequestedCapabilities,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, Default, PartialEq, Eq)]
pub struct SageGrantedNetworkPermissions {
    whitelist: BTreeSet<SageNetworkWhitelistEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, Default, PartialEq, Eq)]
pub struct SageGrantedPermissions {
    capabilities: BTreeSet<UserBridgeCapability>,
    network: SageGrantedNetworkPermissions,
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
    id: String,
    app_id: String,
    app_name: String,
    target: PendingStorageCleanupTarget,
    created_at_ms: i64,
    last_attempt_at_ms: Option<i64>,
    attempt_count: u32,
    last_error: Option<String>,
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

#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
pub struct SageAppPackageManifest {
    name: String,
    version: String,
    permissions: SageRequestedPermissions,
    files: Vec<SageAppManifestFile>,
    entry: Option<String>,
    icon: Option<String>,
    author: Option<SageAppAuthor>,
    donation: Option<SageAppDonation>,
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

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, Default)]
#[serde(rename_all = "camelCase")]
pub struct SageAppFlags {
    pub has_secret_access: bool,
    pub has_external_access: bool,
    pub storage_may_contain_secrets: bool,
    pub isolated: bool,
}

#[allow(clippy::struct_excessive_bools)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[allow(clippy::large_enum_variant)]
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
            .map(|value| value.parse().map_err(serde::de::Error::custom))
            .collect::<Result<Vec<_>, _>>()?;

        let optional_network = raw
            .network
            .whitelist
            .optional
            .into_iter()
            .map(|value| value.parse().map_err(serde::de::Error::custom))
            .collect::<Result<Vec<_>, _>>()?;

        SageRequestedPermissions::new(
            SageRequestedNetworkPermissions::new(required_network, optional_network),
            raw.capabilities.unwrap_or_default(),
        )
        .map_err(serde::de::Error::custom)
    }
}

impl<'de> Deserialize<'de> for SageAppPackageManifest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = <RawSageAppPackageManifest as Deserialize>::deserialize(deserializer)?;

        SageAppPackageManifest::try_from(SageAppPackageManifestParts {
            name: raw.name,
            version: raw.version,
            permissions: raw
                .permissions
                .unwrap_or_else(SageRequestedPermissions::empty),
            files: raw.files,
            entry: raw.entry,
            icon: raw.icon,
            author: raw.author,
            donation: raw.donation,
        })
        .map_err(serde::de::Error::custom)
    }
}

impl Serialize for SageNetworkWhitelistEntry {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.as_permission_string())
    }
}

impl<'de> Deserialize<'de> for SageNetworkWhitelistEntry {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        value.parse().map_err(serde::de::Error::custom)
    }
}

#[derive(Debug)]
pub struct SageAppPackageManifestParts {
    pub name: String,
    pub version: String,
    pub permissions: SageRequestedPermissions,
    pub files: Vec<SageAppManifestFile>,
    pub entry: Option<String>,
    pub icon: Option<String>,
    pub author: Option<SageAppAuthor>,
    pub donation: Option<SageAppDonation>,
}

impl TryFrom<SageAppPackageManifestParts> for SageAppPackageManifest {
    type Error = anyhow::Error;

    fn try_from(value: SageAppPackageManifestParts) -> anyhow::Result<Self> {
        if value.name.trim().is_empty() {
            anyhow::bail!("manifest name cannot be empty");
        }

        if value.version.trim().is_empty() {
            anyhow::bail!("manifest version cannot be empty");
        }

        if let Some(author) = &value.author
            && author.name.trim().is_empty()
        {
            anyhow::bail!("author name cannot be empty");
        }

        if let Some(donation) = &value.donation {
            Self::validate_donation(&donation.address)?;
        }

        Self::validate_files(&value.files)?;

        Ok(Self {
            name: value.name,
            version: value.version,
            permissions: value.permissions,
            files: value.files,
            entry: value.entry,
            icon: value.icon,
            author: value.author,
            donation: value.donation,
        })
    }
}

impl SageAppPackageManifest {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn permissions(&self) -> &SageRequestedPermissions {
        &self.permissions
    }

    pub fn files(&self) -> &[SageAppManifestFile] {
        &self.files
    }

    pub fn entry(&self) -> Option<&str> {
        self.entry.as_deref()
    }

    pub fn icon(&self) -> Option<&str> {
        self.icon.as_deref()
    }

    pub fn author(&self) -> Option<&SageAppAuthor> {
        self.author.as_ref()
    }

    pub fn donation(&self) -> Option<&SageAppDonation> {
        self.donation.as_ref()
    }

    pub fn total_bytes(&self) -> anyhow::Result<u64> {
        Self::compute_total_bytes(&self.files)
    }

    fn validate_donation(address: &str) -> anyhow::Result<()> {
        if address.trim().is_empty() {
            return Err(anyhow!("donation address cannot be empty"));
        }

        if !address.starts_with("xch") && !address.starts_with("txch") {
            return Err(anyhow!("invalid donation address format"));
        }

        Ok(())
    }

    fn validate_files(files: &[SageAppManifestFile]) -> anyhow::Result<()> {
        if files.is_empty() {
            anyhow::bail!("manifest files cannot be empty");
        }

        if files.len() > MAX_APP_FILE_COUNT {
            anyhow::bail!(
                "manifest file count {} exceeds limit {}",
                files.len(),
                MAX_APP_FILE_COUNT
            );
        }

        let mut seen = BTreeSet::new();
        let mut total: u64 = 0;

        for file in files {
            validate_manifest_file_path(&file.path)?;
            validate_sha256_hex(&file.sha256)?;

            if !seen.insert(file.path.clone()) {
                anyhow::bail!("duplicate manifest file path: {}", file.path);
            }

            total = total
                .checked_add(file.size)
                .ok_or_else(|| anyhow!("manifest total size overflow"))?;
        }

        if total > MAX_APP_TOTAL_SIZE_BYTES {
            anyhow::bail!("manifest total size {total} exceeds limit {MAX_APP_TOTAL_SIZE_BYTES}");
        }

        Ok(())
    }

    fn compute_total_bytes(files: &[SageAppManifestFile]) -> anyhow::Result<u64> {
        let mut total: u64 = 0;

        for file in files {
            total = total
                .checked_add(file.size)
                .ok_or_else(|| anyhow!("manifest total size overflow"))?;
        }

        Ok(total)
    }
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

impl PendingStorageCleanupEntry {
    pub fn new(app: &UserSageApp, target: PendingStorageCleanupTarget, error: &str) -> Self {
        let now = unix_timestamp_ms();

        Self {
            id: Uuid::new_v4().to_string(),
            app_id: app.common.id.clone(),
            app_name: app.common.name.clone(),
            target,
            created_at_ms: now,
            last_attempt_at_ms: Some(now),
            attempt_count: 1,
            last_error: Some(error.to_string()),
        }
    }

    pub fn record_failed_attempt(&mut self, error: &str) {
        self.last_attempt_at_ms = Some(unix_timestamp_ms());
        self.attempt_count = self.attempt_count.saturating_add(1);
        self.last_error = Some(error.to_string());
    }

    pub fn target(&self) -> &PendingStorageCleanupTarget {
        &self.target
    }
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

impl SageNetworkWhitelistEntry {
    pub fn new(scheme: impl Into<String>, host: impl Into<String>) -> anyhow::Result<Self> {
        let scheme = scheme.into().trim().to_ascii_lowercase();
        let host = host.into().trim().to_ascii_lowercase();

        if !Self::is_allowed_scheme(&scheme) {
            anyhow::bail!("invalid scheme '{scheme}', only https and wss allowed");
        }

        if host.is_empty()
            || host.contains('/')
            || host.contains('?')
            || host.contains('#')
            || host.contains(' ')
        {
            anyhow::bail!("invalid host in network entry: {scheme}://{host}");
        }
        Ok(Self { scheme, host })
    }

    pub fn scheme(&self) -> &str {
        &self.scheme
    }

    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn as_permission_string(&self) -> String {
        format!("{}://{}", self.scheme, self.host)
    }

    fn is_allowed_scheme(s: &str) -> bool {
        matches!(s, "https" | "wss")
    }

    #[cfg(test)]
    pub fn new_unchecked(scheme: &str, host: &str) -> Self {
        Self::new(scheme, host).unwrap()
    }
}

impl std::str::FromStr for SageNetworkWhitelistEntry {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let value = value.trim();

        let (scheme, host) = value
            .split_once("://")
            .ok_or_else(|| anyhow::anyhow!("invalid network entry, missing scheme: {value}"))?;

        Self::new(scheme, host)
    }
}

impl SageRequestedNetworkWhitelist {
    pub fn new(
        required: impl IntoIterator<Item = SageNetworkWhitelistEntry>,
        optional: impl IntoIterator<Item = SageNetworkWhitelistEntry>,
    ) -> Self {
        let required = required.into_iter().collect::<BTreeSet<_>>();

        let optional = optional
            .into_iter()
            .filter(|entry| !required.contains(entry))
            .collect::<BTreeSet<_>>();

        Self { required, optional }
    }

    pub fn empty() -> Self {
        Self::new([], [])
    }

    pub fn required(&self) -> impl Iterator<Item = &SageNetworkWhitelistEntry> {
        self.required.iter()
    }

    pub fn optional(&self) -> impl Iterator<Item = &SageNetworkWhitelistEntry> {
        self.optional.iter()
    }

    pub fn is_required(&self, entry: &SageNetworkWhitelistEntry) -> bool {
        self.required.contains(entry)
    }

    pub fn is_optional(&self, entry: &SageNetworkWhitelistEntry) -> bool {
        self.optional.contains(entry)
    }

    pub fn is_allowed(&self, entry: &SageNetworkWhitelistEntry) -> bool {
        self.is_required(entry) || self.is_optional(entry)
    }
}

impl SageRequestedCapabilities {
    pub fn new(
        required: impl IntoIterator<Item = UserBridgeCapability>,
        optional: impl IntoIterator<Item = UserBridgeCapability>,
    ) -> Self {
        let required = required.into_iter().collect::<BTreeSet<_>>();

        let optional = optional
            .into_iter()
            .filter(|cap| !required.contains(cap))
            .collect::<BTreeSet<_>>();

        Self { required, optional }
    }

    pub fn empty() -> Self {
        Self::new([], [])
    }

    pub fn required(&self) -> impl Iterator<Item = &UserBridgeCapability> {
        self.required.iter()
    }

    pub fn optional(&self) -> impl Iterator<Item = &UserBridgeCapability> {
        self.optional.iter()
    }

    pub fn is_required(&self, cap: &UserBridgeCapability) -> bool {
        self.required.contains(cap)
    }

    pub fn is_optional(&self, cap: &UserBridgeCapability) -> bool {
        self.optional.contains(cap)
    }

    pub fn is_allowed(&self, cap: &UserBridgeCapability) -> bool {
        self.is_required(cap) || self.is_optional(cap)
    }

    pub fn user_grantable(&self) -> Vec<UserBridgeCapability> {
        self.required()
            .chain(self.optional())
            .copied()
            .filter(|cap| get_user_capability_definition(*cap).flags.user_grantable)
            .collect()
    }

    pub fn resolve_effective_grants(
        &self,
        user_granted: impl IntoIterator<Item = UserBridgeCapability>,
    ) -> anyhow::Result<Vec<UserBridgeCapability>> {
        let user_granted = self.build_user_grants(user_granted)?;

        let mut effective = user_granted;

        for capability in self.required().chain(self.optional()) {
            let definition = get_user_capability_definition(*capability);

            if !definition.flags.user_grantable {
                effective.insert(*capability);
            }
        }

        Ok(effective.into_iter().collect())
    }

    fn build_user_grants(
        &self,
        user_granted: impl IntoIterator<Item = UserBridgeCapability>,
    ) -> anyhow::Result<BTreeSet<UserBridgeCapability>> {
        let user_granted = user_granted.into_iter().collect::<BTreeSet<_>>();

        for capability in &user_granted {
            if !self.is_allowed(capability) {
                anyhow::bail!(
                    "granted capability not requested in manifest: {}",
                    capability.key()
                );
            }

            let definition = get_user_capability_definition(*capability);

            if !definition.flags.user_grantable {
                anyhow::bail!(
                    "granted capability is not user grantable: {}",
                    capability.key()
                );
            }
        }

        for capability in self.required() {
            let definition = get_user_capability_definition(*capability);

            if definition.flags.user_grantable && !user_granted.contains(capability) {
                anyhow::bail!("missing required capability: {}", capability.key());
            }
        }

        Ok(user_granted)
    }
}

impl SageGrantedNetworkPermissions {
    pub fn new(
        requested: &SageRequestedNetworkPermissions,
        whitelist: impl IntoIterator<Item = SageNetworkWhitelistEntry>,
    ) -> anyhow::Result<Self> {
        let whitelist = whitelist.into_iter().collect::<BTreeSet<_>>();

        for entry in &whitelist {
            if !requested.whitelist.is_allowed(entry) {
                anyhow::bail!(
                    "granted network whitelist entry not requested in manifest: {}",
                    entry.as_permission_string()
                );
            }
        }

        Ok(Self { whitelist })
    }

    pub fn whitelist(&self) -> impl Iterator<Item = &SageNetworkWhitelistEntry> {
        self.whitelist.iter()
    }

    pub fn contains(&self, entry: &SageNetworkWhitelistEntry) -> bool {
        self.whitelist.contains(entry)
    }

    pub fn into_vec(self) -> Vec<SageNetworkWhitelistEntry> {
        self.whitelist.into_iter().collect()
    }
}

impl SageGrantedPermissions {
    pub fn new(
        requested: &SageRequestedPermissions,
        capabilities: impl IntoIterator<Item = UserBridgeCapability>,
        network_whitelist: impl IntoIterator<Item = SageNetworkWhitelistEntry>,
    ) -> anyhow::Result<Self> {
        let capabilities = Self::build_capabilities(&requested.capabilities, capabilities)?;

        let network = SageGrantedNetworkPermissions::new(&requested.network, network_whitelist)?;

        let effective_capabilities = requested
            .capabilities
            .resolve_effective_grants(capabilities.iter().copied())?;

        validate_permissions_policy(
            effective_capabilities,
            network.whitelist().cloned(),
            "granted permissions",
        )?;

        Ok(Self {
            capabilities,
            network,
        })
    }

    pub fn from_requested_and_granted(
        requested: &SageRequestedPermissions,
        granted: SageGrantedPermissions,
    ) -> anyhow::Result<Self> {
        Self::new(requested, granted.capabilities, granted.network.whitelist)
    }

    pub fn capabilities(&self) -> impl Iterator<Item = &UserBridgeCapability> {
        self.capabilities.iter()
    }

    pub fn capabilities_vec(&self) -> Vec<UserBridgeCapability> {
        self.capabilities.iter().copied().collect()
    }

    pub fn network(&self) -> &SageGrantedNetworkPermissions {
        &self.network
    }

    pub fn shared_capabilities(&self) -> Vec<UserBridgeCapability> {
        self.capabilities().copied().shared()
    }

    fn build_capabilities(
        requested: &SageRequestedCapabilities,
        capabilities: impl IntoIterator<Item = UserBridgeCapability>,
    ) -> anyhow::Result<BTreeSet<UserBridgeCapability>> {
        let capabilities = capabilities.into_iter().collect::<BTreeSet<_>>();

        for cap in &capabilities {
            if !requested.is_allowed(cap) {
                anyhow::bail!(
                    "granted capability not requested in manifest: {}",
                    cap.key()
                );
            }

            if !get_user_capability_definition(*cap).flags.user_grantable {
                anyhow::bail!("granted capability is not user grantable: {}", cap.key());
            }
        }

        for cap in requested.required() {
            if get_user_capability_definition(*cap).flags.user_grantable
                && !capabilities.contains(cap)
            {
                anyhow::bail!("missing required capability: {}", cap.key());
            }
        }

        Ok(capabilities)
    }

    #[cfg(test)]
    pub fn new_unchecked(
        capabilities: impl IntoIterator<Item = UserBridgeCapability>,
        network_whitelist: impl IntoIterator<Item = SageNetworkWhitelistEntry>,
    ) -> Self {
        Self {
            capabilities: capabilities.into_iter().collect(),
            network: SageGrantedNetworkPermissions {
                whitelist: network_whitelist.into_iter().collect(),
            },
        }
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
            &manifest.permissions,
            granted_permissions,
        )?;

        let effective_capabilities = manifest
            .permissions
            .capabilities
            .resolve_effective_grants(granted_permissions.capabilities().copied())?;

        let capability_flags = get_app_flags(&effective_capabilities, None)?;
        Self::validate_app_flags_policy(capability_flags)?;

        Ok(Self {
            id,
            origin_id,
            name: manifest.name.clone(),
            version: manifest.version.clone(),
            app_dir,
            entry_file: manifest_entry_file(manifest).to_string(),
            icon_file: manifest_icon_file(manifest).to_string(),
            requested_permissions: manifest.permissions.clone(),
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
            &pending.manifest.permissions,
            granted_permissions,
        )?;

        let effective_capabilities = pending
            .manifest
            .permissions
            .capabilities
            .resolve_effective_grants(granted_permissions.capabilities().copied())?;

        let capability_flags =
            get_app_flags(&effective_capabilities, Some(&self.capability_flags))?;

        Self::validate_app_flags_policy(capability_flags)?;

        self.name.clone_from(&pending.manifest.name);
        self.version.clone_from(&pending.manifest.version);
        self.requested_permissions
            .clone_from(&pending.manifest.permissions);
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

impl SageRequestedNetworkPermissions {
    pub fn new(
        required: impl IntoIterator<Item = SageNetworkWhitelistEntry>,
        optional: impl IntoIterator<Item = SageNetworkWhitelistEntry>,
    ) -> Self {
        Self {
            whitelist: SageRequestedNetworkWhitelist::new(required, optional),
        }
    }

    pub fn empty() -> Self {
        Self::new([], [])
    }

    pub fn whitelist(&self) -> &SageRequestedNetworkWhitelist {
        &self.whitelist
    }
}

impl SageRequestedPermissions {
    pub fn new(
        network: SageRequestedNetworkPermissions,
        capabilities: SageRequestedCapabilities,
    ) -> anyhow::Result<Self> {
        for capability in capabilities.required().chain(capabilities.optional()) {
            let definition = get_user_capability_definition(*capability);

            if !definition.flags.requestable_by_app {
                anyhow::bail!(
                    "capability is not requestable by app manifest: {}",
                    capability.key()
                );
            }
        }

        validate_permissions_policy(
            capabilities.required().copied(),
            network.whitelist().required().cloned(),
            "required requested permissions",
        )?;

        Ok(Self {
            network,
            capabilities,
        })
    }

    pub fn empty() -> Self {
        Self {
            network: SageRequestedNetworkPermissions::empty(),
            capabilities: SageRequestedCapabilities::empty(),
        }
    }
}

fn validate_permissions_policy(
    capabilities: impl IntoIterator<Item = UserBridgeCapability>,
    network: impl IntoIterator<Item = SageNetworkWhitelistEntry>,
    context: &str,
) -> anyhow::Result<()> {
    let capability_flags = capabilities
        .into_iter()
        .fold(CapabilityFlags::EMPTY, |flags, cap| {
            flags.union(get_user_capability_definition(cap).flags)
        });

    let has_secret_access = capability_flags.accesses_sensitive_secret;
    let has_external_access =
        capability_flags.externally_observable || network.into_iter().next().is_some();

    if has_secret_access && has_external_access {
        anyhow::bail!("{context} cannot include both external access and sensitive secret access");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::bridge::capabilities::UserBridgeCapability;
    use crate::permissions::user_registry;
    use crate::types::{
        SageGrantedPermissions, SageNetworkWhitelistEntry, SageRequestedCapabilities,
        SageRequestedNetworkPermissions, SageRequestedPermissions,
    };

    #[test]
    fn granted_permissions_rejects_non_user_grantable_capability_as_user_grant() {
        let auto = UserBridgeCapability::AppGetInfo;

        let requested = SageRequestedPermissions::new(
            SageRequestedNetworkPermissions::empty(),
            SageRequestedCapabilities::new([auto], []),
        )
        .expect("requested permissions should be valid");

        let err = SageGrantedPermissions::new(&requested, [auto], [])
            .expect_err("non-user-grantable capability cannot be persisted as user grant");

        assert!(
            err.to_string().contains("not user grantable"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn non_user_grantable_requested_capability_is_effective_without_user_grant() {
        let auto = UserBridgeCapability::AppGetInfo;

        let requested = SageRequestedCapabilities::new([auto], []);

        let effective = requested
            .resolve_effective_grants([])
            .expect("auto capability should still be effective");

        assert_eq!(effective, vec![auto]);
    }

    #[test]
    fn effective_grants_include_non_user_grantable_requested_capability() {
        let auto = UserBridgeCapability::AppGetInfo;

        let optional_requested = SageRequestedCapabilities::new([], [auto]);
        assert_eq!(
            optional_requested.resolve_effective_grants([]).unwrap(),
            vec![auto]
        );

        let required_requested = SageRequestedCapabilities::new([auto], []);
        assert_eq!(
            required_requested.resolve_effective_grants([]).unwrap(),
            vec![auto]
        );
    }

    #[test]
    fn effective_grants_do_not_include_removed_non_user_grantable_capability() {
        let auto = UserBridgeCapability::AppGetInfo;

        let requested = SageRequestedCapabilities::new([auto], []);
        assert_eq!(requested.resolve_effective_grants([]).unwrap(), vec![auto]);

        let removed_requested = SageRequestedCapabilities::new([], []);
        assert!(
            removed_requested
                .resolve_effective_grants([])
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn rejects_non_requestable_required_capability() {
        let non_requestable = first_non_requestable_capability();

        let err = SageRequestedPermissions::new(
            SageRequestedNetworkPermissions::empty(),
            SageRequestedCapabilities::new([non_requestable], []),
        )
        .expect_err("expected non-requestable required capability to be rejected");

        let message = err.to_string();
        assert!(message.contains(non_requestable.key()));
    }

    #[test]
    fn rejects_non_requestable_optional_capability() {
        let non_requestable = first_non_requestable_capability();

        let err = SageRequestedPermissions::new(
            SageRequestedNetworkPermissions::empty(),
            SageRequestedCapabilities::new([], [non_requestable]),
        )
        .expect_err("expected non-requestable optional capability to be rejected");

        let message = err.to_string();
        assert!(message.contains(non_requestable.key()));
    }

    #[test]
    fn requested_capabilities_deduplicates_and_removes_required_from_optional() {
        let requested = SageRequestedCapabilities::new(
            [
                UserBridgeCapability::WalletSendXch,
                UserBridgeCapability::WalletSendXch,
            ],
            [UserBridgeCapability::WalletSendXch],
        );

        assert_eq!(
            requested.required().copied().collect::<Vec<_>>(),
            vec![UserBridgeCapability::WalletSendXch]
        );

        assert!(requested.optional().next().is_none());
    }

    #[test]
    fn requested_network_deduplicates_and_removes_required_from_optional() {
        let requested = SageRequestedPermissions::new(
            SageRequestedNetworkPermissions::new(
                [
                    SageNetworkWhitelistEntry::new("HTTPS", "Example.com").unwrap(),
                    SageNetworkWhitelistEntry::new("https", "example.com").unwrap(),
                ],
                [
                    SageNetworkWhitelistEntry::new("WSS", "ws.example.com").unwrap(),
                    SageNetworkWhitelistEntry::new("https", "example.com").unwrap(),
                ],
            ),
            SageRequestedCapabilities::empty(),
        )
        .unwrap();

        let required = requested
            .network
            .whitelist
            .required()
            .cloned()
            .collect::<Vec<_>>();

        let optional = requested
            .network
            .whitelist
            .optional()
            .cloned()
            .collect::<Vec<_>>();

        assert_eq!(
            required,
            vec![SageNetworkWhitelistEntry::new("https", "example.com").unwrap()]
        );

        assert_eq!(
            optional,
            vec![SageNetworkWhitelistEntry::new("wss", "ws.example.com").unwrap()]
        );
    }

    #[test]
    fn granted_permissions_rejects_unrequested_capability() {
        let requested = SageRequestedPermissions::new(
            SageRequestedNetworkPermissions::empty(),
            SageRequestedCapabilities::new([UserBridgeCapability::WalletSendXch], []),
        )
        .unwrap();

        let err = SageGrantedPermissions::new(
            &requested,
            [
                UserBridgeCapability::WalletSendXch,
                UserBridgeCapability::PersistentStorage,
            ],
            [],
        )
        .expect_err("expected unrequested capability to be rejected");

        assert!(err.to_string().contains("persistent_storage"));
    }

    #[test]
    fn granted_permissions_rejects_missing_required_user_grantable_capability() {
        let requested = SageRequestedPermissions::new(
            SageRequestedNetworkPermissions::empty(),
            SageRequestedCapabilities::new([UserBridgeCapability::WalletSendXch], []),
        )
        .unwrap();

        let err = SageGrantedPermissions::new(&requested, [], [])
            .expect_err("expected missing required capability to be rejected");

        assert!(
            err.to_string()
                .contains(UserBridgeCapability::WalletSendXch.key())
        );
    }

    #[test]
    fn granted_permissions_allows_subset_of_optional_capabilities() {
        let requested = SageRequestedPermissions::new(
            SageRequestedNetworkPermissions::empty(),
            SageRequestedCapabilities::new(
                [UserBridgeCapability::WalletSendXch],
                [UserBridgeCapability::PersistentStorage],
            ),
        )
        .unwrap();

        SageGrantedPermissions::new(&requested, [UserBridgeCapability::WalletSendXch], [])
            .expect("expected optional capability to be omittable");
    }

    #[test]
    fn non_user_grantable_required_capability_is_effective_without_persisted_grant() {
        let auto = UserBridgeCapability::AppGetInfo;

        let requested = SageRequestedCapabilities::new([auto], []);

        let effective = requested
            .resolve_effective_grants([])
            .expect("expected effective permissions to resolve");

        assert_eq!(effective, vec![auto]);
    }

    #[test]
    fn non_user_grantable_optional_capability_is_effective_without_persisted_grant() {
        let auto = UserBridgeCapability::AppGetInfo;

        let requested = SageRequestedCapabilities::new([], [auto]);

        let effective = requested
            .resolve_effective_grants([])
            .expect("expected effective permissions to resolve");

        assert_eq!(effective, vec![auto]);
    }

    #[test]
    fn user_grantable_required_capability_without_user_grant_is_blocked() {
        let requested = SageRequestedCapabilities::new([UserBridgeCapability::WalletSendXch], []);

        let err = requested
            .resolve_effective_grants([])
            .expect_err("required user-grantable capability should require user grant");

        assert!(
            err.to_string()
                .contains(UserBridgeCapability::WalletSendXch.key())
        );
    }

    #[test]
    fn requested_permissions_policy_rejects_required_secret_and_external_combination() {
        let err = SageRequestedPermissions::new(
            SageRequestedNetworkPermissions::empty(),
            SageRequestedCapabilities::new(
                [
                    UserBridgeCapability::WalletSendXch,
                    UserBridgeCapability::WalletGetSecretKey,
                ],
                [],
            ),
        )
        .expect_err("expected incompatible requested capability policy to be rejected");

        assert!(
            err.to_string().contains(
                "required requested permissions cannot include both external access and sensitive secret access"
            ),
            "unexpected error: {err}"
        );
    }

    fn first_non_requestable_capability() -> UserBridgeCapability {
        user_registry()
            .values()
            .find(|definition| !definition.flags.requestable_by_app)
            .unwrap_or_else(|| {
                panic!("test requires at least one capability with requestable_by_app = false")
            })
            .capability
    }
}

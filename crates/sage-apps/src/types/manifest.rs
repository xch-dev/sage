use serde::{Deserialize, Deserializer, Serialize};
use specta::Type;
use std::path::Path;

use crate::types::app::{SageAppAuthor, SageAppDonation};
use crate::types::invariants::{
    normalize_optional_manifest_path, validate_declared_manifest_asset_exists,
    validate_manifest_file_path, validate_manifest_files, validate_package_files_match_manifest,
    validate_sha256_hex,
};
use crate::types::normalizers::normalized_non_empty_string;
use crate::types::permissions::SageRequestedPermissions;

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
pub struct SageAppManifestFile {
    path: String,
    sha256: String,
    size: u64,
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

#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
pub struct SageAppPackageManifest {
    name: String,
    version: String,
    permissions: SageRequestedPermissions,
    files: Vec<SageAppManifestFile>,
    total_bytes: u64,
    entry: Option<String>,
    icon: Option<String>,
    author: Option<SageAppAuthor>,
    donation: Option<SageAppDonation>,
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

impl TryFrom<SageAppPackageManifestParts> for SageAppPackageManifest {
    type Error = anyhow::Error;

    fn try_from(value: SageAppPackageManifestParts) -> anyhow::Result<Self> {
        let name = normalized_non_empty_string(value.name, "manifest name")?;
        let version = normalized_non_empty_string(value.version, "manifest version")?;

        let entry = normalize_optional_manifest_path(value.entry, "manifest entry")?;
        let icon = normalize_optional_manifest_path(value.icon, "manifest icon")?;

        let author = value
            .author
            .map(|author| SageAppAuthor::new(author.name(), author.avatar()))
            .transpose()?;

        let donation = value
            .donation
            .map(|donation| SageAppDonation::new(donation.address()))
            .transpose()?;

        let total_bytes = validate_manifest_files(&value.files)?;

        validate_declared_manifest_asset_exists(entry.as_deref(), &value.files, "entry")?;
        validate_declared_manifest_asset_exists(icon.as_deref(), &value.files, "icon")?;

        Ok(Self {
            name,
            version,
            permissions: value.permissions,
            files: value.files,
            total_bytes,
            entry,
            icon,
            author,
            donation,
        })
    }
}

impl SageAppPackageManifest {
    pub fn validate_package_files(&self, package_root: &Path) -> anyhow::Result<()> {
        validate_package_files_match_manifest(package_root, self.files())
    }

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

    pub fn total_bytes(&self) -> u64 {
        self.total_bytes
    }

    pub fn entry(&self) -> &str {
        self.entry.as_deref().unwrap_or("index.html")
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
}

impl SageAppManifestFile {
    pub fn new(
        path: impl Into<String>,
        sha256: impl Into<String>,
        size: u64,
    ) -> anyhow::Result<Self> {
        let path = normalized_non_empty_string(path, "manifest file path")?;
        validate_manifest_file_path(&path)?;

        let sha256 = normalized_non_empty_string(sha256, "manifest file sha256")?;
        validate_sha256_hex(&sha256)?;

        Ok(Self { path, sha256, size })
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn sha256(&self) -> &str {
        &self.sha256
    }

    pub fn size(&self) -> u64 {
        self.size
    }
}


#[cfg(test)]
mod tests {
    use crate::bridge::capabilities::UserBridgeCapability;
    use crate::types::{
        SageAppManifestFile, SageAppPackageManifest, SageAppPackageManifestParts,
        SageNetworkWhitelistEntry, SageRequestedCapabilities, SageRequestedNetworkPermissions,
        SageRequestedPermissions,
    };

    fn sample_manifest_file(path: &str, size: u64) -> SageAppManifestFile {
        SageAppManifestFile::new(path.to_string(), "a".repeat(64), size).unwrap()
    }

    fn sample_file() -> SageAppManifestFile {
        sample_manifest_file("index.html", 123)
    }

    fn entry(scheme: &str, host: &str) -> SageNetworkWhitelistEntry {
        SageNetworkWhitelistEntry::new(scheme, host).unwrap()
    }

    fn requested_permissions() -> SageRequestedPermissions {
        SageRequestedPermissions::new(
            SageRequestedNetworkPermissions::new(
                [entry("https", "required.example.com")],
                [entry("wss", "optional.example.com")],
            ),
            SageRequestedCapabilities::new(
                [UserBridgeCapability::WalletSendXch],
                [
                    UserBridgeCapability::PersistentStorage,
                    UserBridgeCapability::WalletGetSecretKey,
                ],
            ),
        )
            .unwrap()
    }

    fn sample_manifest_with(
        entry_file: Option<String>,
        icon_file: Option<String>,
    ) -> SageAppPackageManifest {
        let mut files = vec![sample_manifest_file("index.html", 1)];

        if let Some(entry_file) = &entry_file
            && entry_file != "index.html"
        {
            files.push(sample_manifest_file(entry_file, 1));
        }

        if let Some(icon_file) = &icon_file {
            files.push(sample_manifest_file(icon_file, 1));
        }

        SageAppPackageManifest::try_from(SageAppPackageManifestParts {
            name: "Test App".to_string(),
            version: "1.0.0".to_string(),
            permissions: requested_permissions(),
            files,
            entry: entry_file,
            icon: icon_file,
            author: None,
            donation: None,
        })
            .unwrap()
    }

    #[test]
    fn manifest_rejects_blank_name() {
        let err = SageAppPackageManifest::try_from(SageAppPackageManifestParts {
            name: "   ".to_string(),
            version: "1.0.0".to_string(),
            permissions: SageRequestedPermissions::empty(),
            files: vec![sample_file()],
            entry: Some("index.html".to_string()),
            icon: Some("icon.png".to_string()),
            author: None,
            donation: None,
        })
            .unwrap_err();

        assert!(err.to_string().contains("name cannot be empty"));
    }

    #[test]
    fn manifest_rejects_blank_version() {
        let err = SageAppPackageManifest::try_from(SageAppPackageManifestParts {
            name: "Test".to_string(),
            version: "   ".to_string(),
            permissions: SageRequestedPermissions::empty(),
            files: vec![sample_file()],
            entry: Some("index.html".to_string()),
            icon: Some("icon.png".to_string()),
            author: None,
            donation: None,
        })
            .unwrap_err();

        assert!(err.to_string().contains("version cannot be empty"));
    }

    #[test]
    fn manifest_total_size_is_computed() {
        let manifest = SageAppPackageManifest::try_from(SageAppPackageManifestParts {
            name: "Test App".to_string(),
            version: "1.0.0".to_string(),
            permissions: SageRequestedPermissions::empty(),
            files: vec![sample_manifest_file("dist/index.html", 123)],
            entry: Some("dist/index.html".to_string()),
            icon: None,
            author: None,
            donation: None,
        })
            .unwrap();

        assert_eq!(manifest.total_bytes(), 123);
    }

    #[test]
    fn manifest_entry_file_uses_explicit_entry() {
        let manifest =
            sample_manifest_with(Some("entry.html".to_string()), Some("icon.svg".to_string()));

        assert_eq!(manifest.entry(), "entry.html");
    }

    #[test]
    fn manifest_entry_file_defaults_to_index_html() {
        let manifest = sample_manifest_with(None, Some("icon.svg".to_string()));
        assert_eq!(manifest.entry(), "index.html");
    }

    #[test]
    fn manifest_icon_file_uses_explicit_icon() {
        let manifest =
            sample_manifest_with(Some("entry.html".to_string()), Some("icon.svg".to_string()));

        assert_eq!(manifest.icon().unwrap(), "icon.svg");
    }

    #[test]
    fn manifest_icon_file_defaults_to_none() {
        let manifest = sample_manifest_with(Some("entry.html".to_string()), None);
        assert_eq!(manifest.icon(), None);
    }
}


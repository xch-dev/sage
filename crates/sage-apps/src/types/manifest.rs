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

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, Type, PartialEq, Eq)]
pub struct SageAppManifestVersion(pub u16);

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SageAppManifestSageVersion {
    pub min: String,

    #[serde(default)]
    pub tested_max: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SageAppManifestHeaderV0 {
    #[serde(default)]
    pub manifest_version: SageAppManifestVersion,

    pub name: String,

    #[serde(default)]
    pub icon: Option<String>,

    pub sage_version: SageAppManifestSageVersion,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
pub struct SageAppManifestFile {
    path: String,
    sha256: String,
    size: u64,
}

#[derive(Debug)]
pub struct SageAppPackageManifestParts {
    pub manifest_version: SageAppManifestVersion,
    pub name: String,
    pub icon: Option<String>,
    pub sage_version: SageAppManifestSageVersion,
    pub version: String,
    pub permissions: SageRequestedPermissions,
    pub files: Vec<SageAppManifestFile>,
    pub entry: Option<String>,
    pub author: Option<SageAppAuthor>,
    pub donation: Option<SageAppDonation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[allow(clippy::large_enum_variant)]
pub enum SageAppPackageManifestPreview {
    Full {
        manifest: SageAppPackageManifest,
    },
    Partial {
        manifest_header: SageAppManifestHeaderV0,
        parse_error: String,
    },
}

#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SageAppPackageManifest {
    manifest_version: SageAppManifestVersion,
    name: String,
    icon: Option<String>,
    sage_version: SageAppManifestSageVersion,
    version: String,
    permissions: SageRequestedPermissions,
    files: Vec<SageAppManifestFile>,
    total_bytes: u64,
    entry: Option<String>,
    author: Option<SageAppAuthor>,
    donation: Option<SageAppDonation>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawSageAppPackageManifest {
    #[serde(default)]
    manifest_version: SageAppManifestVersion,

    name: String,

    #[serde(default)]
    icon: Option<String>,

    sage_version: SageAppManifestSageVersion,

    version: String,

    #[serde(default)]
    permissions: Option<SageRequestedPermissions>,

    #[serde(default)]
    files: Vec<SageAppManifestFile>,

    #[serde(default)]
    entry: Option<String>,

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
            manifest_version: raw.manifest_version,
            name: raw.name,
            icon: raw.icon,
            sage_version: raw.sage_version,
            version: raw.version,
            permissions: raw
                .permissions
                .unwrap_or_else(SageRequestedPermissions::empty),
            files: raw.files,
            entry: raw.entry,
            author: raw.author,
            donation: raw.donation,
        })
        .map_err(serde::de::Error::custom)
    }
}

impl TryFrom<SageAppPackageManifestParts> for SageAppPackageManifest {
    type Error = anyhow::Error;

    fn try_from(value: SageAppPackageManifestParts) -> anyhow::Result<Self> {
        if value.manifest_version.0 != 0 {
            return Err(anyhow::anyhow!(
                "unsupported manifestVersion {}",
                value.manifest_version.0
            ));
        }

        let name = normalized_non_empty_string(value.name, "manifest name")?;
        let version = normalized_non_empty_string(value.version, "manifest version")?;

        let sage_version_min =
            normalized_non_empty_string(value.sage_version.min, "manifest sageVersion.min")?;

        let sage_version_tested_max = value
            .sage_version
            .tested_max
            .map(|tested_max| {
                normalized_non_empty_string(tested_max, "manifest sageVersion.testedMax")
            })
            .transpose()?;

        let sage_version = SageAppManifestSageVersion {
            min: sage_version_min,
            tested_max: sage_version_tested_max,
        };

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
            manifest_version: value.manifest_version,
            name,
            icon,
            sage_version,
            version,
            permissions: value.permissions,
            files: value.files,
            total_bytes,
            entry,
            author,
            donation,
        })
    }
}

impl SageAppPackageManifestPreview {
    pub fn full_manifest(&self) -> Option<&SageAppPackageManifest> {
        match self {
            Self::Full { manifest } => Some(manifest),
            Self::Partial { .. } => None,
        }
    }

    pub fn manifest_header(&self) -> SageAppManifestHeaderV0 {
        match self {
            Self::Full { manifest } => manifest.header_v0(),
            Self::Partial {
                manifest_header, ..
            } => manifest_header.clone(),
        }
    }

    pub fn parse_error(&self) -> Option<&str> {
        match self {
            Self::Full { .. } => None,
            Self::Partial { parse_error, .. } => Some(parse_error),
        }
    }
}

impl SageAppPackageManifestParts {
    pub fn v0_defaults() -> (SageAppManifestVersion, SageAppManifestSageVersion) {
        (
            SageAppManifestVersion(0),
            SageAppManifestSageVersion {
                min: "0.0.0".to_string(),
                tested_max: None,
            },
        )
    }
}

impl SageAppPackageManifest {
    pub fn validate_package_files(&self, package_root: &Path) -> anyhow::Result<()> {
        validate_package_files_match_manifest(package_root, self.files())
    }

    pub fn manifest_version(&self) -> SageAppManifestVersion {
        self.manifest_version
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn sage_version(&self) -> &SageAppManifestSageVersion {
        &self.sage_version
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

    pub fn header_v0(&self) -> SageAppManifestHeaderV0 {
        SageAppManifestHeaderV0 {
            manifest_version: self.manifest_version,
            name: self.name.clone(),
            icon: self.icon.clone(),
            sage_version: self.sage_version.clone(),
        }
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

pub fn parse_manifest_version_from_value(
    value: &serde_json::Value,
) -> anyhow::Result<SageAppManifestVersion> {
    let version = value
        .get("manifestVersion")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| anyhow::anyhow!("manifestVersion is missing"))?;

    let version =
        u16::try_from(version).map_err(|_| anyhow::anyhow!("manifestVersion is too large"))?;

    Ok(SageAppManifestVersion(version))
}

pub fn parse_manifest_header_v0_from_value(
    value: serde_json::Value,
) -> anyhow::Result<SageAppManifestHeaderV0> {
    let version = parse_manifest_version_from_value(&value)?;

    if version.0 != 0 {
        return Err(anyhow::anyhow!(
            "unsupported manifest header version {}",
            version.0
        ));
    }

    serde_json::from_value(value)
        .map_err(|err| anyhow::anyhow!("failed to parse manifest v0 header: {err}"))
}

#[cfg(test)]
mod tests {
    use crate::capabilities::list::UserBridgeCapability;
    use crate::types::{
        SageAppManifestFile, SageAppManifestSageVersion, SageAppManifestVersion,
        SageAppPackageManifest, SageAppPackageManifestParts, SageNetworkWhitelistEntry,
        SageRequestedCapabilities, SageRequestedNetworkPermissions, SageRequestedPermissions,
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
                    UserBridgeCapability::StoragePersistentWebview,
                    UserBridgeCapability::WalletGetSecretKey,
                ],
            ),
        )
        .unwrap()
    }

    fn manifest_header_parts() -> (SageAppManifestVersion, SageAppManifestSageVersion) {
        (
            SageAppManifestVersion(0),
            SageAppManifestSageVersion {
                min: "0.0.0".to_string(),
                tested_max: None,
            },
        )
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

        let (manifest_version, sage_version) = manifest_header_parts();

        SageAppPackageManifest::try_from(SageAppPackageManifestParts {
            manifest_version,
            name: "Test App".to_string(),
            icon: icon_file,
            sage_version,
            version: "1.0.0".to_string(),
            permissions: requested_permissions(),
            files,
            entry: entry_file,
            author: None,
            donation: None,
        })
        .unwrap()
    }

    #[test]
    fn manifest_rejects_blank_name() {
        let (manifest_version, sage_version) = manifest_header_parts();

        let err = SageAppPackageManifest::try_from(SageAppPackageManifestParts {
            manifest_version,
            name: "   ".to_string(),
            icon: Some("icon.png".to_string()),
            sage_version,
            version: "1.0.0".to_string(),
            permissions: SageRequestedPermissions::empty(),
            files: vec![sample_file()],
            entry: Some("index.html".to_string()),
            author: None,
            donation: None,
        })
        .unwrap_err();

        assert!(err.to_string().contains("name cannot be empty"));
    }

    #[test]
    fn manifest_rejects_blank_version() {
        let (manifest_version, sage_version) = manifest_header_parts();

        let err = SageAppPackageManifest::try_from(SageAppPackageManifestParts {
            manifest_version,
            name: "Test".to_string(),
            icon: Some("icon.png".to_string()),
            sage_version,
            version: "   ".to_string(),
            permissions: SageRequestedPermissions::empty(),
            files: vec![sample_file()],
            entry: Some("index.html".to_string()),
            author: None,
            donation: None,
        })
        .unwrap_err();

        assert!(err.to_string().contains("version cannot be empty"));
    }

    #[test]
    fn manifest_rejects_unsupported_manifest_version() {
        let (_, sage_version) = manifest_header_parts();

        let err = SageAppPackageManifest::try_from(SageAppPackageManifestParts {
            manifest_version: SageAppManifestVersion(1),
            name: "Test".to_string(),
            icon: None,
            sage_version,
            version: "1.0.0".to_string(),
            permissions: SageRequestedPermissions::empty(),
            files: vec![sample_file()],
            entry: Some("index.html".to_string()),
            author: None,
            donation: None,
        })
        .unwrap_err();

        assert!(err.to_string().contains("unsupported manifestVersion 1"));
    }

    #[test]
    fn manifest_total_size_is_computed() {
        let (manifest_version, sage_version) = manifest_header_parts();

        let manifest = SageAppPackageManifest::try_from(SageAppPackageManifestParts {
            manifest_version,
            name: "Test App".to_string(),
            icon: None,
            sage_version,
            version: "1.0.0".to_string(),
            permissions: SageRequestedPermissions::empty(),
            files: vec![sample_manifest_file("dist/index.html", 123)],
            entry: Some("dist/index.html".to_string()),
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

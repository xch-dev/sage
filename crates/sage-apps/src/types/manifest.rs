use std::collections::BTreeSet;

use anyhow::anyhow;
use serde::{Deserialize, Deserializer, Serialize};
use specta::Type;

use crate::lifecycle::{
    validate_manifest_file_path, validate_sha256_hex, MAX_APP_FILE_COUNT,
    MAX_APP_TOTAL_SIZE_BYTES,
};
use crate::types::app::{SageAppAuthor, SageAppDonation};
use crate::types::normalizers::{normalized_non_empty_string, normalized_optional_string};
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
            permissions: raw.permissions.unwrap_or_else(SageRequestedPermissions::empty),
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

        let total_bytes = Self::validate_files(&value.files)?;

        Self::validate_declared_asset_exists(entry.as_deref(), &value.files, "entry")?;
        Self::validate_declared_asset_exists(icon.as_deref(), &value.files, "icon")?;

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

    fn validate_declared_asset_exists(
        path: Option<&str>,
        files: &[SageAppManifestFile],
        label: &str,
    ) -> anyhow::Result<()> {
        let Some(path) = path else {
            return Ok(());
        };

        if !files.iter().any(|file| file.path == path) {
            anyhow::bail!("manifest {label} file is not listed in files: {path}");
        }

        Ok(())
    }

    fn validate_files(files: &[SageAppManifestFile]) -> anyhow::Result<u64> {
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
            validate_manifest_file_path(file.path())?;
            validate_sha256_hex(file.sha256())?;

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

        Ok(total)
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

fn normalize_optional_manifest_path(
    path: Option<String>,
    label: &str,
) -> anyhow::Result<Option<String>> {
    let path = normalized_optional_string(path);

    if let Some(path) = &path {
        validate_manifest_file_path(path)
            .map_err(|err| anyhow!("{label} is invalid: {err}"))?;
    }

    Ok(path)
}

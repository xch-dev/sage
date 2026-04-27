use std::collections::BTreeSet;
use anyhow::anyhow;
use serde::{Deserialize, Deserializer, Serialize};
use specta::Type;
use crate::lifecycle::{validate_manifest_file_path, validate_sha256_hex, MAX_APP_FILE_COUNT, MAX_APP_TOTAL_SIZE_BYTES};
use crate::types::app::{SageAppAuthor, SageAppDonation};
use crate::types::permissions::SageRequestedPermissions;

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
pub struct SageAppManifestFile {
    pub path: String,
    pub sha256: String,
    pub size: u64,
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

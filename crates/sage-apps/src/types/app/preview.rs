use serde::{Deserialize, Serialize};
use specta::Type;

use crate::types::{
    SageAppPackageManifest, SageAppPackageManifestPreview, SageAppUrl,
};
use crate::types::normalizers::normalized_non_empty_string;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct UserSageAppPendingUpdate {
    app_url: SageAppUrl,
    manifest_hash: String,
    manifest: SageAppPackageManifest,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SageAppUrlPreview {
    app_url: SageAppUrl,
    manifest_hash: String,
    manifest: SageAppPackageManifestPreview,
}

impl UserSageAppPendingUpdate {
    pub fn new(
        app_url: SageAppUrl,
        manifest_hash: String,
        manifest: SageAppPackageManifest,
    ) -> Self {
        Self {
            app_url,
            manifest_hash,
            manifest,
        }
    }

    pub fn app_url(&self) -> &SageAppUrl {
        &self.app_url
    }

    pub fn manifest_hash(&self) -> &str {
        &self.manifest_hash
    }

    pub fn manifest(&self) -> &SageAppPackageManifest {
        &self.manifest
    }
}

impl SageAppUrlPreview {
    pub fn new(
        app_url: &SageAppUrl,
        manifest: SageAppPackageManifestPreview,
        manifest_hash: String,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            app_url: app_url.clone(),
            manifest_hash: normalized_non_empty_string(manifest_hash, "manifest hash")?,
            manifest,
        })
    }

    pub fn from_full_manifest(
        app_url: &SageAppUrl,
        manifest: SageAppPackageManifest,
        manifest_hash: String,
    ) -> anyhow::Result<Self> {
        Self::new(
            app_url,
            SageAppPackageManifestPreview::Full { manifest },
            manifest_hash,
        )
    }

    pub fn from_pending_update(pending: &UserSageAppPendingUpdate) -> Self {
        Self {
            app_url: pending.app_url().clone(),
            manifest_hash: pending.manifest_hash().to_string(),
            manifest: SageAppPackageManifestPreview::Full {
                manifest: pending.manifest().clone(),
            },
        }
    }

    pub fn app_url(&self) -> &SageAppUrl {
        &self.app_url
    }

    pub fn manifest_hash(&self) -> &str {
        &self.manifest_hash
    }

    pub fn manifest(&self) -> &SageAppPackageManifestPreview {
        &self.manifest
    }

    pub fn full_manifest(&self) -> Option<&SageAppPackageManifest> {
        self.manifest.full_manifest()
    }

    pub fn require_full_manifest(&self) -> anyhow::Result<&SageAppPackageManifest> {
        self.full_manifest().ok_or_else(|| {
            anyhow::anyhow!(
                "manifest could not be fully parsed: {}",
                self.manifest
                    .parse_error()
                    .unwrap_or("unknown manifest parse error")
            )
        })
    }
}

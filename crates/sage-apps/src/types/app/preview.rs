use serde::{Deserialize, Serialize};
use specta::Type;

use crate::types::manifest::SageAppPackageManifest;
use crate::types::normalizers::normalized_non_empty_string;
use crate::types::{SageAppManifestUrl, SageAppUrl};

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct UserSageAppPendingUpdate {
    app_url: String,
    manifest_url: String,
    manifest_hash: String,
    manifest: SageAppPackageManifest,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SageAppUrlPreview {
    app_url: String,
    manifest_url: String,
    manifest_hash: String,
    manifest: SageAppPackageManifest,
}

impl UserSageAppPendingUpdate {
    pub fn new(
        app_url: String,
        manifest_url: String,
        manifest_hash: String,
        manifest: SageAppPackageManifest,
    ) -> Self {
        Self {
            app_url,
            manifest_url,
            manifest_hash,
            manifest,
        }
    }

    pub fn app_url(&self) -> &str {
        &self.app_url
    }

    pub fn manifest_url(&self) -> &str {
        &self.manifest_url
    }

    pub fn manifest_hash(&self) -> &str {
        &self.manifest_hash
    }

    pub fn manifest(&self) -> &SageAppPackageManifest {
        &self.manifest
    }
}

impl SageAppUrlPreview {
    pub async fn new(app_url: impl Into<String>) -> anyhow::Result<Self> {
        let app_url = crate::lifecycle::install::url::normalize_app_url(&app_url.into())?;
        let app_url = SageAppUrl::parse(app_url)?;
        let manifest_url = SageAppManifestUrl::derive_from_app_url(&app_url)?;
        let (manifest, manifest_hash) = crate::lifecycle::fetch_url_manifest(&manifest_url).await?;

        Ok(Self {
            app_url: app_url.into_string(),
            manifest_url: manifest_url.into_string(),
            manifest_hash: normalized_non_empty_string(manifest_hash, "manifest hash")?,
            manifest,
        })
    }

    pub fn from_pending_update(pending: &UserSageAppPendingUpdate) -> Self {
        Self {
            app_url: pending.app_url().to_string(),
            manifest_url: pending.manifest_url().to_string(),
            manifest_hash: pending.manifest_hash().to_string(),
            manifest: pending.manifest().clone(),
        }
    }

    pub fn app_url(&self) -> &str {
        &self.app_url
    }

    pub fn manifest_url(&self) -> &str {
        &self.manifest_url
    }

    pub fn manifest_hash(&self) -> &str {
        &self.manifest_hash
    }

    pub fn manifest(&self) -> &SageAppPackageManifest {
        &self.manifest
    }
}

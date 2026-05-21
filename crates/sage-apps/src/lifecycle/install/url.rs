use std::path::{Path, PathBuf};

use anyhow::Result as AnyResult;
use async_trait::async_trait;

use super::AppInstallSource;
use crate::lifecycle::registry::read_installed_app_by_id;
use crate::lifecycle::{download_url_snapshot, fetch_url_manifest_preview};
use crate::types::{
    SageAppPackageManifest, SageAppSnapshot, SageAppUrl, SageAppUrlPreview, UserSageApp,
    UserSageAppSource,
};
use crate::utils::bytes_sha256_hex;

#[derive(Debug, Clone)]
pub struct PreparedUrlInstall {
    pub preview: SageAppUrlPreview,
}

#[async_trait]
impl AppInstallSource for SageAppUrl {
    type PreparedArtifact = PreparedUrlInstall;

    async fn prepare(&self) -> AnyResult<Self::PreparedArtifact> {
        let (manifest, manifest_hash) = fetch_url_manifest_preview(&self.manifest_url()).await?;

        Ok(PreparedUrlInstall {
            preview: SageAppUrlPreview::new(self, manifest, manifest_hash).await?,
        })
    }

    fn manifest<'a>(&self, prepared: &'a Self::PreparedArtifact) -> &'a SageAppPackageManifest {
        prepared
            .preview
            .require_full_manifest()
            .expect("URL install requires full manifest")
    }

    fn source(&self, prepared: &Self::PreparedArtifact) -> UserSageAppSource {
        UserSageAppSource::Url {
            app_url: prepared.preview.app_url().clone(),
        }
    }

    fn resolve_target(
        &self,
        root: &Path,
        _base_path: &Path,
        prepared: &Self::PreparedArtifact,
    ) -> AnyResult<(String, PathBuf, Option<UserSageApp>)> {
        resolve_url_install_target(root, prepared.preview.app_url())
    }

    async fn create_snapshot(
        &self,
        app_dir: &Path,
        prepared: &Self::PreparedArtifact,
    ) -> AnyResult<SageAppSnapshot> {
        download_url_snapshot(
            app_dir,
            prepared.preview.app_url(),
            prepared.preview.require_full_manifest()?,
            prepared.preview.manifest_hash(),
        )
            .await
    }
}

pub fn generate_url_app_id(app_url: &SageAppUrl) -> String {
    let hash = bytes_sha256_hex(app_url.manifest_url().as_bytes());
    format!("url-{}-{}", app_url.slug(), &hash[..16])
}

pub fn resolve_url_install_target(
    root: &Path,
    app_url: &SageAppUrl,
) -> AnyResult<(String, PathBuf, Option<UserSageApp>)> {
    let app_id = generate_url_app_id(app_url);
    let app_dir = root.join(&app_id);

    let existing = if app_dir.exists() {
        Some(read_installed_app_by_id(
            root.parent().unwrap_or(root),
            &app_id,
        )?)
    } else {
        None
    };

    Ok((app_id.clone(), app_dir, existing))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn generate_url_app_id_is_stable_for_same_app_url() {
        let a = generate_url_app_id(&SageAppUrl::parse("https://example.com/app").unwrap());
        let b = generate_url_app_id(&SageAppUrl::parse("https://example.com/app").unwrap());

        assert_eq!(a, b);
        assert!(a.starts_with("url-example-com-"));
    }

    #[test]
    fn generate_url_app_id_differs_for_different_manifest_urls() {
        let a = generate_url_app_id(&SageAppUrl::parse("https://example.com/app-a").unwrap());
        let b = generate_url_app_id(&SageAppUrl::parse("https://example.com/app-b").unwrap());

        assert_ne!(a, b);
    }

    #[test]
    fn resolve_url_install_target_returns_stable_app_id_and_dir() {
        let dir = tempdir().unwrap();
        let root = dir.path().join("apps");
        std::fs::create_dir_all(&root).unwrap();

        let app_url = SageAppUrl::parse("https://example.com/app").unwrap();
        let (app_id, app_dir, existing) = resolve_url_install_target(&root, &app_url).unwrap();

        assert!(existing.is_none());
        assert_eq!(app_dir, root.join(&app_id));
        assert_eq!(app_id, generate_url_app_id(&app_url));
    }
}

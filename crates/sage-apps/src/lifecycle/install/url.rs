use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result as AnyResult};
use async_trait::async_trait;
use url::Url;
use uuid::Uuid;

use super::AppInstallSource;
use crate::lifecycle::{
    derive_manifest_url, download_url_snapshot, fetch_url_manifest, read_retired_app_origins,
    write_retired_app_origins,
};
use crate::lifecycle::registry::read_installed_app_by_id;
use crate::types::{
    SageAppPackageManifest, SageAppSnapshot, SageAppUrlPreview, UserSageApp, UserSageAppSource,
};
use crate::utils::{bytes_sha256_hex, slugify_app_name};

#[derive(Debug, Clone)]
pub struct UrlInstallSource {
    pub app_url: String,
}

#[derive(Debug, Clone)]
pub struct PreparedUrlInstall {
    pub preview: SageAppUrlPreview,
}

#[async_trait]
impl AppInstallSource for UrlInstallSource {
    type Prepared = PreparedUrlInstall;

    async fn prepare(&self) -> AnyResult<Self::Prepared> {
        Ok(PreparedUrlInstall {
            preview: preview_app_url_internal(self.app_url.clone()).await?,
        })
    }

    fn manifest<'a>(&self, prepared: &'a Self::Prepared) -> &'a SageAppPackageManifest {
        &prepared.preview.manifest
    }

    fn source(&self, prepared: &Self::Prepared) -> UserSageAppSource {
        UserSageAppSource::Url {
            app_url: prepared.preview.app_url.clone(),
            manifest_url: prepared.preview.manifest_url.clone(),
        }
    }

    fn resolve_target(
        &self,
        root: &Path,
        _base_path: &Path,
        prepared: &Self::Prepared,
    ) -> AnyResult<(String, PathBuf, Option<UserSageApp>)> {
        resolve_url_install_target(root, &prepared.preview.manifest_url)
    }

    async fn create_snapshot(
        &self,
        app_dir: &Path,
        prepared: &Self::Prepared,
    ) -> AnyResult<SageAppSnapshot> {
        download_url_snapshot(
            app_dir,
            &prepared.preview.app_url,
            &prepared.preview.manifest,
            &prepared.preview.manifest_hash,
        )
            .await
    }

    fn origin_id(
        &self,
        base_path: &Path,
        app_id: &str,
        existing: Option<&UserSageApp>,
    ) -> AnyResult<String> {
        if let Some(existing) = existing {
            return Ok(existing.common.origin_id.clone());
        }

        if should_rotate_url_origin_on_install(base_path, app_id)? {
            Ok(generate_rotated_url_origin_id(app_id))
        } else {
            Ok(default_url_origin_id(app_id))
        }
    }

    fn after_origin_selected(
        &self,
        base_path: &Path,
        app_id: &str,
        origin_id: &str,
    ) -> AnyResult<()> {
        clear_pending_cleanup_for_reused_url_origin(base_path, app_id, origin_id)
    }
}

pub async fn preview_app_url_internal(app_url: String) -> AnyResult<SageAppUrlPreview> {
    let app_url = normalize_app_url(&app_url)?;

    let manifest_url = derive_manifest_url(&app_url)?;
    let (manifest, manifest_hash) = fetch_url_manifest(&manifest_url).await?;

    Ok(SageAppUrlPreview {
        app_url,
        manifest_url,
        manifest_hash,
        manifest,
    })
}

pub fn normalize_app_url(url: &str) -> AnyResult<String> {
    let mut parsed = Url::parse(url).context("invalid app URL")?;

    let scheme = parsed.scheme();
    let host = parsed
        .host_str()
        .ok_or_else(|| anyhow!("app URL is missing host"))?
        .to_ascii_lowercase();

    let is_local_dev_host = host == "localhost" || host == "127.0.0.1" || host == "::1";

    match scheme {
        "https" => {}
        "http" if is_local_dev_host => {}
        "http" => {
            return Err(anyhow!(
                "URL app install requires HTTPS, except for localhost/127.0.0.1 development URLs"
            ));
        }
        other => {
            return Err(anyhow!("unsupported app URL scheme: {other}"));
        }
    }

    parsed.set_fragment(None);

    let path = parsed.path();
    if !path.ends_with('/') {
        parsed.set_path(&format!("{path}/"));
    }

    if parsed.query().is_some() {
        parsed.set_query(None);
    }

    Ok(parsed.to_string())
}

pub fn generate_url_app_id(manifest_url: &str) -> String {
    let hash = bytes_sha256_hex(manifest_url.as_bytes());
    format!("url-{}-{}", slugify_host(manifest_url), &hash[..16])
}

pub fn default_url_origin_id(app_id: &str) -> String {
    app_id.to_string()
}

pub fn generate_rotated_url_origin_id(app_id: &str) -> String {
    let suffix = Uuid::new_v4().simple().to_string();
    format!("r{}-{}", &suffix[..12], app_id)
}

pub fn should_rotate_url_origin_on_install(base_path: &Path, app_id: &str) -> AnyResult<bool> {
    let retired = read_retired_app_origins(base_path)?;

    Ok(retired
        .iter()
        .any(|entry| entry.app_id == app_id && entry.storage_may_contain_secrets))
}

pub fn clear_pending_cleanup_for_reused_url_origin(
    base_path: &Path,
    app_id: &str,
    origin_id: &str,
) -> AnyResult<()> {
    let mut retired = read_retired_app_origins(base_path)?;
    let mut changed = false;

    for entry in &mut retired {
        if entry.app_id == app_id && entry.origin_id == origin_id && entry.cleanup_pending {
            entry.cleanup_pending = false;
            changed = true;
        }
    }

    if changed {
        write_retired_app_origins(base_path, &retired)?;
    }

    Ok(())
}

pub fn resolve_url_install_target(
    root: &Path,
    manifest_url: &str,
) -> AnyResult<(String, PathBuf, Option<UserSageApp>)> {
    let app_id = generate_url_app_id(manifest_url);
    let app_dir = root.join(&app_id);

    let existing = if app_dir.exists() {
        Some(read_installed_app_by_id(root.parent().unwrap_or(root), &app_id)?)
    } else {
        None
    };

    Ok((app_id.clone(), app_dir, existing))
}

fn slugify_host(input: &str) -> String {
    if let Ok(url) = Url::parse(input) {
        if let Some(host) = url.host_str() {
            return slugify_app_name(host);
        }
    }

    slugify_app_name(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bridge::capabilities::UserBridgeCapability;
    use crate::lifecycle::write_retired_app_origins;
    use crate::types::{InstalledSageAppStorage, RetiredAppOriginEntry, SageAppCommon, SageAppManifestFile, SageAppPackageManifestParts, SageGrantedPermissions, SageNetworkWhitelistEntry, SageRequestedCapabilities, SageRequestedNetworkPermissions, SageRequestedPermissions};
    use tempfile::{tempdir, TempDir};

    fn fake_retired_app_origins(dir: &TempDir, storage_may_contain_secrets: bool) {
        write_retired_app_origins(
            dir.path(),
            &[RetiredAppOriginEntry {
                id: "retired-1".into(),
                app_id: "url-abc123".into(),
                app_name: "Test App".into(),
                origin_id: "url-abc123".into(),
                created_at_ms: 1,
                storage_may_contain_secrets,
                cleanup_pending: false,
            }],
        )
            .unwrap();
    }

    fn sample_manifest() -> SageAppPackageManifest {
        let permissions = SageRequestedPermissions::new(
            SageRequestedNetworkPermissions::new(
                [],
                [SageNetworkWhitelistEntry::new("https", "api.example.com").unwrap()],
            ),
            SageRequestedCapabilities::new(
                [],
                [UserBridgeCapability::PersistentStorage],
            ),
        )
            .unwrap();

        SageAppPackageManifest::try_from(SageAppPackageManifestParts {
            name: "Test App".into(),
            version: "1.0.0".into(),
            permissions,
            files: vec![SageAppManifestFile {
                path: "index.html".into(),
                sha256: "a".repeat(64),
                size: 123,
            }],
            entry: Some("index.html".into()),
            icon: Some("icon.png".into()),
            author: None,
            donation: None,
        })
            .unwrap()
    }

    fn sample_app(app_id: &str, origin_id: &str) -> UserSageApp {
        let dir = tempdir().unwrap();
        let app_dir = dir.path().join(app_id);
        std::fs::create_dir_all(&app_dir).unwrap();

        let manifest = sample_manifest();

        let granted_permissions = SageGrantedPermissions::new(
            manifest.permissions(),
            [UserBridgeCapability::PersistentStorage],
            [SageNetworkWhitelistEntry::new_unchecked("https", "api.example.com")],
        )
            .unwrap();

        let snapshot = SageAppSnapshot {
            manifest_hash: "hash".into(),
            snapshot_dir: app_dir.to_string_lossy().to_string(),
            total_bytes: 123,
            manifest: manifest.clone(),
        };

        let common = SageAppCommon::new(
            app_id.into(),
            origin_id.into(),
            app_dir.to_string_lossy().to_string(),
            &manifest,
            granted_permissions,
            InstalledSageAppStorage::Unmanaged,
            snapshot,
        )
            .unwrap();

        UserSageApp {
            common,
            source: UserSageAppSource::Url {
                app_url: "https://example.com/app/".into(),
                manifest_url: "https://example.com/app/sage-manifest.json".into(),
            },
            pending_update: None,
        }
    }

    #[test]
    fn normalize_app_url_keeps_https_and_adds_trailing_slash() {
        let out = normalize_app_url("https://example.com/app").unwrap();
        assert_eq!(out, "https://example.com/app/");
    }

    #[test]
    fn normalize_app_url_strips_query_and_fragment() {
        let out = normalize_app_url("https://example.com/app?x=1#frag").unwrap();
        assert_eq!(out, "https://example.com/app/");
    }

    #[test]
    fn normalize_app_url_allows_localhost_http() {
        let out = normalize_app_url("http://localhost:4173").unwrap();
        assert_eq!(out, "http://localhost:4173/");
    }

    #[test]
    fn normalize_app_url_rejects_non_local_http() {
        let err = normalize_app_url("http://example.com/app")
            .unwrap_err()
            .to_string();

        assert!(err.contains("requires HTTPS"));
    }

    #[test]
    fn generate_url_app_id_is_stable_for_same_manifest_url() {
        let a = generate_url_app_id("https://example.com/app/sage-manifest.json");
        let b = generate_url_app_id("https://example.com/app/sage-manifest.json");

        assert_eq!(a, b);
        assert!(a.starts_with("url-example-com-"));
    }

    #[test]
    fn default_url_origin_id_is_same_as_app_id() {
        assert_eq!(default_url_origin_id("url-abc123"), "url-abc123");
    }

    #[test]
    fn rotated_url_origin_id_differs_from_app_id() {
        let origin = generate_rotated_url_origin_id("url-abc123");

        assert_ne!(origin, "url-abc123");
        assert!(origin.ends_with("url-abc123"));
        assert!(origin.starts_with('r'));
    }

    #[test]
    fn should_rotate_url_origin_on_install_is_false_without_retired_entry() {
        let dir = tempdir().unwrap();

        assert!(!should_rotate_url_origin_on_install(dir.path(), "url-abc123").unwrap());
    }

    #[test]
    fn should_not_rotate_url_origin_for_pending_cleanup_without_secrets() {
        let dir = tempdir().unwrap();

        fake_retired_app_origins(&dir, false);

        assert!(!should_rotate_url_origin_on_install(dir.path(), "url-abc123").unwrap());
    }

    #[test]
    fn should_not_rotate_url_origin_for_clean_retired_origin() {
        let dir = tempdir().unwrap();

        fake_retired_app_origins(&dir, false);

        assert!(!should_rotate_url_origin_on_install(dir.path(), "url-abc123").unwrap());
    }

    #[test]
    fn should_rotate_url_origin_when_retired_storage_may_contain_secrets() {
        let dir = tempdir().unwrap();

        write_retired_app_origins(
            dir.path(),
            &[RetiredAppOriginEntry {
                id: "retired-1".into(),
                app_id: "url-abc123".into(),
                app_name: "Test App".into(),
                origin_id: "url-abc123".into(),
                created_at_ms: 1,
                storage_may_contain_secrets: true,
                cleanup_pending: false,
            }],
        )
            .unwrap();

        assert!(should_rotate_url_origin_on_install(dir.path(), "url-abc123").unwrap());
    }

    #[test]
    fn should_rotate_url_origin_when_retired_storage_may_contain_secrets_even_if_cleanup_pending() {
        let dir = tempdir().unwrap();

        fake_retired_app_origins(&dir, true);

        assert!(should_rotate_url_origin_on_install(dir.path(), "url-abc123").unwrap());
    }

    #[test]
    fn url_origin_id_reuses_existing_origin() {
        let dir = tempdir().unwrap();
        let existing = sample_app("url-abc123", "existing-origin");

        let source = UrlInstallSource {
            app_url: "https://example.com/app/".into(),
        };

        let origin = source
            .origin_id(dir.path(), "url-abc123", Some(&existing))
            .unwrap();

        assert_eq!(origin, "existing-origin");
    }

    #[test]
    fn url_origin_id_defaults_to_app_id_without_retired_origin() {
        let dir = tempdir().unwrap();

        let source = UrlInstallSource {
            app_url: "https://example.com/app/".into(),
        };

        let origin = source.origin_id(dir.path(), "url-abc123", None).unwrap();

        assert_eq!(origin, "url-abc123");
    }

    #[test]
    fn url_origin_id_reuses_default_origin_for_pending_cleanup_without_secrets() {
        let dir = tempdir().unwrap();

        fake_retired_app_origins(&dir, false);

        let source = UrlInstallSource {
            app_url: "https://example.com/app/".into(),
        };

        let origin = source.origin_id(dir.path(), "url-abc123", None).unwrap();

        assert_eq!(origin, "url-abc123");
    }

    #[test]
    fn url_origin_id_rotates_with_retired_secret_storage() {
        let dir = tempdir().unwrap();

        fake_retired_app_origins(&dir, true);

        let source = UrlInstallSource {
            app_url: "https://example.com/app/".into(),
        };

        let origin = source.origin_id(dir.path(), "url-abc123", None).unwrap();

        assert_ne!(origin, "url-abc123");
        assert!(origin.ends_with("url-abc123"));
        assert!(origin.starts_with('r'));
    }

    #[test]
    fn reused_url_origin_clears_pending_cleanup_after_origin_selected() {
        let dir = tempdir().unwrap();

        write_retired_app_origins(
            dir.path(),
            &[RetiredAppOriginEntry {
                id: "retired-1".into(),
                app_id: "url-abc123".into(),
                app_name: "Test App".into(),
                origin_id: "url-abc123".into(),
                created_at_ms: 1,
                storage_may_contain_secrets: false,
                cleanup_pending: true,
            }],
        )
            .unwrap();

        let before = read_retired_app_origins(dir.path()).unwrap();
        assert!(before[0].cleanup_pending);

        let source = UrlInstallSource {
            app_url: "https://example.com/app/".into(),
        };

        let origin = source.origin_id(dir.path(), "url-abc123", None).unwrap();
        assert_eq!(origin, "url-abc123");

        source
            .after_origin_selected(dir.path(), "url-abc123", &origin)
            .unwrap();

        let after = read_retired_app_origins(dir.path()).unwrap();
        assert_eq!(after.len(), 1);
        assert!(!after[0].cleanup_pending);
        assert!(!after[0].storage_may_contain_secrets);
    }

    #[test]
    fn rotated_url_origin_does_not_clear_pending_cleanup_for_old_origin() {
        let dir = tempdir().unwrap();

        write_retired_app_origins(
            dir.path(),
            &[RetiredAppOriginEntry {
                id: "retired-1".into(),
                app_id: "url-abc123".into(),
                app_name: "Test App".into(),
                origin_id: "url-abc123".into(),
                created_at_ms: 1,
                storage_may_contain_secrets: true,
                cleanup_pending: true,
            }],
        )
            .unwrap();

        let source = UrlInstallSource {
            app_url: "https://example.com/app/".into(),
        };

        let rotated = source.origin_id(dir.path(), "url-abc123", None).unwrap();
        assert_ne!(rotated, "url-abc123");

        source
            .after_origin_selected(dir.path(), "url-abc123", &rotated)
            .unwrap();

        let retired = read_retired_app_origins(dir.path()).unwrap();
        assert_eq!(retired.len(), 1);
        assert!(retired[0].cleanup_pending);
        assert!(retired[0].storage_may_contain_secrets);
    }
}

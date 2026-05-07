use std::path::{Path, PathBuf};

use anyhow::Result as AnyResult;
use async_trait::async_trait;
use uuid::Uuid;

use super::AppInstallSource;
use crate::lifecycle::registry::read_installed_app_by_id;
use crate::lifecycle::{download_url_snapshot, fetch_url_manifest_preview, read_retired_app_origins, write_retired_app_origins};
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
    type Prepared = PreparedUrlInstall;

    async fn prepare(&self) -> AnyResult<Self::Prepared> {
        let (manifest, manifest_hash) = fetch_url_manifest_preview(&self.manifest_url()).await?;

        Ok(PreparedUrlInstall {
            preview: SageAppUrlPreview::new(self, manifest, manifest_hash).await?,
        })
    }

    fn manifest<'a>(&self, prepared: &'a Self::Prepared) -> &'a SageAppPackageManifest {
        prepared
            .preview
            .require_full_manifest()
            .expect("URL install requires full manifest")
    }

    fn source(&self, prepared: &Self::Prepared) -> UserSageAppSource {
        UserSageAppSource::Url {
            app_url: prepared.preview.app_url().clone(),
        }
    }

    fn resolve_target(
        &self,
        root: &Path,
        _base_path: &Path,
        prepared: &Self::Prepared,
    ) -> AnyResult<(String, PathBuf, Option<UserSageApp>)> {
        resolve_url_install_target(root, prepared.preview.app_url())
    }

    async fn create_snapshot(
        &self,
        app_dir: &Path,
        prepared: &Self::Prepared,
    ) -> AnyResult<SageAppSnapshot> {
        download_url_snapshot(
            app_dir,
            prepared.preview.app_url(),
            prepared.preview.require_full_manifest()?,
            prepared.preview.manifest_hash(),
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
            return Ok(existing.common().origin_id().to_string());
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

pub fn generate_url_app_id(app_url: &SageAppUrl) -> String {
    let hash = bytes_sha256_hex(app_url.manifest_url().as_bytes());
    format!("url-{}-{}", app_url.slug(), &hash[..16])
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
        .any(|entry| entry.app_id() == app_id && entry.storage_may_contain_secrets()))
}

pub fn clear_pending_cleanup_for_reused_url_origin(
    base_path: &Path,
    app_id: &str,
    origin_id: &str,
) -> AnyResult<()> {
    let mut retired = read_retired_app_origins(base_path)?;
    let mut changed = false;

    for entry in &mut retired {
        if entry.matches_app_origin(app_id, origin_id) {
            changed |= entry.clear_pending_cleanup();
        }
    }

    if changed {
        write_retired_app_origins(base_path, &retired)?;
    }

    Ok(())
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
    use crate::capabilities::list::UserBridgeCapability;
    use crate::lifecycle::write_retired_app_origins;
    use crate::runtime::{SageAppRuntimeMode, SageAppRuntimeRecord, SageAppRuntimeVisibility};
    use crate::types::{AppPresentation, InstalledSageAppStorage, RetiredAppOriginEntry, SageAppCommon, SageAppIdentity, SageAppManifestFile, SageAppPackageManifestParts, SageAppWalletScope, SageGrantedPermissions, SageNetworkWhitelistEntry, SageRequestedCapabilities, SageRequestedNetworkPermissions, SageRequestedPermissions, SharedSageApp};
    use tempfile::{TempDir, tempdir};

    fn fake_retired_app_origins(
        dir: &TempDir,
        storage_may_contain_secrets: bool,
        cleanup_pending: bool,
    ) {
        let app = sample_app_in(
            dir.path(),
            "url-abc123",
            "url-abc123",
            storage_may_contain_secrets,
        );

        write_retired_app_origins(
            dir.path(),
            &[RetiredAppOriginEntry::new(&app, cleanup_pending)],
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
                [
                    UserBridgeCapability::StoragePersistentWebview,
                    UserBridgeCapability::WalletGetSecretKey,
                ],
            ),
        )
        .unwrap();

        let (manifest_version, sage_version) = SageAppPackageManifestParts::v0_defaults();
        SageAppPackageManifest::try_from(SageAppPackageManifestParts {
            manifest_version,
            name: "Test App".into(),
            icon: None,
            sage_version,
            version: "1.0.0".into(),
            permissions,
            files: vec![SageAppManifestFile::new("index.html", "a".repeat(64), 123).unwrap()],
            entry: Some("index.html".into()),
            author: None,
            donation: None,
        })
        .unwrap()
    }

    fn sample_app_in(
        base: &Path,
        app_id: &str,
        origin_id: &str,
        storage_may_contain_secrets: bool,
    ) -> SharedSageApp {
        let app_dir = crate::lifecycle::registry::app_dir(base, app_id);
        std::fs::create_dir_all(&app_dir).unwrap();
        std::fs::write(app_dir.join("index.html"), "x").unwrap();

        let manifest = sample_manifest();

        let granted_capabilities = if storage_may_contain_secrets {
            vec![
                UserBridgeCapability::StoragePersistentWebview,
                UserBridgeCapability::WalletGetSecretKey,
            ]
        } else {
            vec![UserBridgeCapability::StoragePersistentWebview]
        };

        let granted_permissions =
            SageGrantedPermissions::new(manifest.permissions(), granted_capabilities, []).unwrap();

        let snapshot =
            SageAppSnapshot::new("hash", app_dir.to_string_lossy().to_string(), manifest).unwrap();

        let mut common = SageAppCommon::new(
            SageAppIdentity::new(app_id, origin_id, app_dir.to_string_lossy().to_string()).unwrap(),
            granted_permissions,
            InstalledSageAppStorage::Unmanaged,
            snapshot,
            SageAppWalletScope::AllWallets
        )
            .unwrap();

        if storage_may_contain_secrets {
            common.mark_storage_may_contain_secrets();
        }

        let app = SharedSageApp::new(
            UserSageApp::new_installed(
                common,
                UserSageAppSource::url("https://example.com/app/").unwrap(),
            )
                .into_sage_app(),
        );

        if storage_may_contain_secrets {
            let _runtime = SageAppRuntimeRecord::new(
                &app,
                "test",
                "sage-app://test/index.html",
                AppPresentation::Taskbar,
                SageAppRuntimeMode::Inline,
                SageAppRuntimeVisibility::Visible,
                false,
            ).unwrap();
        }

        app
    }

    #[test]
    fn generate_url_app_id_is_stable_for_same_app_url() {
        let a = generate_url_app_id(&SageAppUrl::parse("https://example.com/app").unwrap());
        let b = generate_url_app_id(&SageAppUrl::parse("https://example.com/app").unwrap());

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

        fake_retired_app_origins(&dir, false, false);

        assert!(!should_rotate_url_origin_on_install(dir.path(), "url-abc123").unwrap());
    }

    #[test]
    fn should_not_rotate_url_origin_for_clean_retired_origin() {
        let dir = tempdir().unwrap();

        fake_retired_app_origins(&dir, false, false);

        assert!(!should_rotate_url_origin_on_install(dir.path(), "url-abc123").unwrap());
    }

    #[test]
    fn should_rotate_url_origin_when_retired_storage_may_contain_secrets() {
        let dir = tempdir().unwrap();

        fake_retired_app_origins(&dir, true, false);

        assert!(should_rotate_url_origin_on_install(dir.path(), "url-abc123").unwrap());
    }

    #[test]
    fn should_rotate_url_origin_when_retired_storage_may_contain_secrets_even_if_cleanup_pending() {
        let dir = tempdir().unwrap();

        fake_retired_app_origins(&dir, true, true);

        assert!(should_rotate_url_origin_on_install(dir.path(), "url-abc123").unwrap());
    }

    #[test]
    fn url_origin_id_reuses_existing_origin() {
        let dir = tempdir().unwrap();
        let existing = sample_app_in(dir.path(), "url-abc123", "existing-origin", false);

        let source = SageAppUrl::parse("https://example.com/app/").unwrap();

        let origin = existing
            .try_with(|app| {
                let user_app = app
                    .as_user()
                    .ok_or_else(|| anyhow::anyhow!("expected user app"))?;

                source.origin_id(dir.path(), "url-abc123", Some(user_app))
            })
            .unwrap();

        assert_eq!(origin, "existing-origin");
    }

    #[test]
    fn url_origin_id_defaults_to_app_id_without_retired_origin() {
        let dir = tempdir().unwrap();

        let source = SageAppUrl::parse("https://example.com/app/").unwrap();

        let origin = source.origin_id(dir.path(), "url-abc123", None).unwrap();

        assert_eq!(origin, "url-abc123");
    }

    #[test]
    fn url_origin_id_reuses_default_origin_for_pending_cleanup_without_secrets() {
        let dir = tempdir().unwrap();

        fake_retired_app_origins(&dir, false, false);

        let source = SageAppUrl::parse("https://example.com/app/").unwrap();

        let origin = source.origin_id(dir.path(), "url-abc123", None).unwrap();

        assert_eq!(origin, "url-abc123");
    }

    #[test]
    fn url_origin_id_rotates_with_retired_secret_storage() {
        let dir = tempdir().unwrap();

        fake_retired_app_origins(&dir, true, false);

        let source = SageAppUrl::parse("https://example.com/app/").unwrap();

        let origin = source.origin_id(dir.path(), "url-abc123", None).unwrap();

        assert_ne!(origin, "url-abc123");
        assert!(origin.ends_with("url-abc123"));
        assert!(origin.starts_with('r'));
    }

    #[test]
    fn reused_url_origin_clears_pending_cleanup_after_origin_selected() {
        let dir = tempdir().unwrap();

        fake_retired_app_origins(&dir, false, true);

        let before = read_retired_app_origins(dir.path()).unwrap();
        assert!(before[0].cleanup_pending());

        let source = SageAppUrl::parse("https://example.com/app/").unwrap();

        let origin = source.origin_id(dir.path(), "url-abc123", None).unwrap();
        assert_eq!(origin, "url-abc123");

        source
            .after_origin_selected(dir.path(), "url-abc123", &origin)
            .unwrap();

        let after = read_retired_app_origins(dir.path()).unwrap();
        assert_eq!(after.len(), 1);
        assert!(!after[0].cleanup_pending());
        assert!(!after[0].storage_may_contain_secrets());
    }

    #[test]
    fn rotated_url_origin_does_not_clear_pending_cleanup_for_old_origin() {
        let dir = tempdir().unwrap();

        fake_retired_app_origins(&dir, true, true);

        let source = SageAppUrl::parse("https://example.com/app/").unwrap();

        let rotated = source.origin_id(dir.path(), "url-abc123", None).unwrap();
        assert_ne!(rotated, "url-abc123");

        source
            .after_origin_selected(dir.path(), "url-abc123", &rotated)
            .unwrap();

        let retired = read_retired_app_origins(dir.path()).unwrap();
        assert_eq!(retired.len(), 1);
        assert!(retired[0].cleanup_pending());
        assert!(retired[0].storage_may_contain_secrets());
    }
}

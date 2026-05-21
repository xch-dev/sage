use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::Result as AnyResult;
use async_trait::async_trait;
use tauri::{AppHandle, Manager, State};
use uuid::Uuid;

use crate::AppsHostState;
use crate::bridge::methods::system::emit_listed_apps_changed;
use crate::lifecycle::{allocate_new_storage, apps_root};
use crate::types::{
    SageAppCommon, SageAppIdentity, SageAppPackageManifest, SageAppSnapshot,
    SageAppStorage, SageAppWalletScope, SageGrantedPermissionsInput, UserSageApp,
    UserSageAppSource,
};

pub mod commands;
pub mod url;
pub mod zip;

#[async_trait]
pub trait AppInstallSource {
    type PreparedArtifact: Send + Sync;

    async fn prepare(&self) -> AnyResult<Self::PreparedArtifact>;

    fn manifest<'a>(&self, prepared: &'a Self::PreparedArtifact) -> &'a SageAppPackageManifest;

    fn source(&self, prepared: &Self::PreparedArtifact) -> UserSageAppSource;

    fn resolve_target(
        &self,
        root: &Path,
        base_path: &Path,
        prepared: &Self::PreparedArtifact,
    ) -> AnyResult<(String, PathBuf)>;

    async fn create_snapshot(
        &self,
        app_dir: &Path,
        prepared: &Self::PreparedArtifact,
    ) -> AnyResult<SageAppSnapshot>;
}

#[derive(Debug)]
struct ResolvedInstallTarget {
    app_id: String,
    app_dir: PathBuf,
    origin_id: String,
    storage: SageAppStorage,
}

pub async fn install_app_from_source<S>(
    app: &AppHandle,
    host_state: &State<'_, AppsHostState>,
    base_path: &Path,
    granted_permissions_input: SageGrantedPermissionsInput,
    wallet_scope: SageAppWalletScope,
    source: S,
) -> AnyResult<UserSageApp>
where
    S: AppInstallSource + Send + Sync,
{
    let install_result = install_app_from_source_inner(
        app,
        host_state,
        base_path,
        granted_permissions_input,
        wallet_scope,
        source,
    )
        .await;

    let host_state: State<'_, AppsHostState> = app.state();
    emit_listed_apps_changed(app, &host_state).await;

    install_result
}

async fn install_app_from_source_inner<S>(
    app: &AppHandle,
    host_state: &State<'_, AppsHostState>,
    base_path: &Path,
    granted_permissions_input: SageGrantedPermissionsInput,
    wallet_scope: SageAppWalletScope,
    source: S,
) -> AnyResult<UserSageApp>
where
    S: AppInstallSource + Send + Sync,
{
    let root = apps_root(base_path);
    fs::create_dir_all(&root)?;

    let prepared_artifact = source.prepare().await?;

    let (app_id, app_dir) =
        source.resolve_target(&root, base_path, &prepared_artifact)?;

    if host_state.db.app_exists(&app_id).await? {
        anyhow::bail!("App is already installed");
    }

    let registered_storage = allocate_new_storage(app, host_state, base_path).await?;
    let origin_id = fresh_origin_id(&app_id);

    let target = ResolvedInstallTarget {
        app_id: app_id.clone(),
        app_dir,
        origin_id: origin_id.clone(),
        storage: registered_storage.storage.clone(),
    };

    let installed = materialize_installed_app(
        source,
        prepared_artifact,
        target,
        granted_permissions_input,
        wallet_scope,
    )
        .await?;

    let mut tx = host_state.db.begin_immediate().await?;

    let origin_row_id = tx
        .register_origin(&origin_id, registered_storage.storage_id)
        .await?;

    tx.insert_user_app(&installed, registered_storage.storage_id, origin_row_id)
        .await?;

    let reread = tx.load_user_app(&app_id).await?;

    let expected = serde_json::to_value(&installed)?;
    let actual = serde_json::to_value(&reread)?;

    if expected != actual {
        tx.rollback().await;
        anyhow::bail!("installed app DB round-trip mismatch");
    }

    tx.commit().await?;

    Ok(installed)
}

async fn materialize_installed_app<S>(
    source: S,
    prepared_artifact: S::PreparedArtifact,
    target: ResolvedInstallTarget,
    granted_permissions_input: SageGrantedPermissionsInput,
    wallet_scope: SageAppWalletScope,
) -> AnyResult<UserSageApp>
where
    S: AppInstallSource + Send + Sync,
{
    let manifest = source.manifest(&prepared_artifact);

    create_app_dir(&target.app_dir)?;

    let snapshot = source
        .create_snapshot(&target.app_dir, &prepared_artifact)
        .await?;

    let granted_permissions = granted_permissions_input.resolve(manifest.permissions())?;

    let common = SageAppCommon::new(
        SageAppIdentity::new(
            target.app_id,
            target.origin_id,
            target.app_dir.to_string_lossy().to_string(),
        )?,
        granted_permissions,
        target.storage,
        snapshot,
        wallet_scope,
    )?;

    Ok(UserSageApp::new_installed(
        common,
        source.source(&prepared_artifact),
    ))
}

#[cfg(test)]
pub(crate) async fn install_app_from_source_for_test<S>(
    base_path: &Path,
    granted_permissions_input: SageGrantedPermissionsInput,
    source: S,
) -> AnyResult<UserSageApp>
where
    S: AppInstallSource + Send + Sync,
{
    let root = apps_root(base_path);
    fs::create_dir_all(&root)?;

    let prepared_artifact = source.prepare().await?;

    let (app_id, app_dir) =
        source.resolve_target(&root, base_path, &prepared_artifact)?;

    materialize_installed_app(
        source,
        prepared_artifact,
        ResolvedInstallTarget {
            app_id: app_id.clone(),
            app_dir,
            origin_id: fresh_origin_id(&app_id),
            storage: SageAppStorage::Unmanaged,
        },
        granted_permissions_input,
        SageAppWalletScope::AllWallets,
    )
        .await
}

pub fn fresh_origin_id(app_id: &str) -> String {
    format!("{}.{}", Uuid::new_v4(), app_id)
}

pub fn create_app_dir(app_dir: &Path) -> AnyResult<()> {
    if app_dir.exists() {
        anyhow::bail!("app directory already exists, cannot create");
    }

    fs::create_dir_all(app_dir)?;

    Ok(())
}

#[cfg(test)]
pub(crate) struct FakeInstallSource {
    pub manifest: SageAppPackageManifest,
    pub app_id: String,
    pub source: UserSageAppSource,
}

#[cfg(test)]
pub(crate) struct FakePreparedArtifact {
    manifest: SageAppPackageManifest,
}

#[async_trait]
#[cfg(test)]
impl AppInstallSource for FakeInstallSource {
    type PreparedArtifact = FakePreparedArtifact;

    async fn prepare(&self) -> AnyResult<Self::PreparedArtifact> {
        Ok(FakePreparedArtifact {
            manifest: self.manifest.clone(),
        })
    }

    fn manifest<'a>(&self, prepared: &'a Self::PreparedArtifact) -> &'a SageAppPackageManifest {
        &prepared.manifest
    }

    fn source(&self, _prepared: &Self::PreparedArtifact) -> UserSageAppSource {
        self.source.clone()
    }

    fn resolve_target(
        &self,
        root: &Path,
        _base_path: &Path,
        _prepared: &Self::PreparedArtifact,
    ) -> AnyResult<(String, PathBuf)> {
        let app_dir = root.join(&self.app_id);
        Ok((self.app_id.clone(), app_dir))
    }

    async fn create_snapshot(
        &self,
        app_dir: &Path,
        prepared: &Self::PreparedArtifact,
    ) -> AnyResult<SageAppSnapshot> {
        fs::write(app_dir.join("index.html"), "x")?;

        Ok(SageAppSnapshot::new(
            "fake-hash",
            app_dir.to_string_lossy().to_string(),
            prepared.manifest.clone(),
        )?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    use tempfile::tempdir;

    use crate::capabilities::list::UserBridgeCapability;
    use crate::types::{
        SageAppManifestFile, SageAppPackageManifestParts, SageGrantedPermissionsInput,
        SageNetworkWhitelistEntry, SageRequestedCapabilities, SageRequestedNetworkPermissions,
        SageRequestedPermissions,
    };

    fn sample_manifest() -> SageAppPackageManifest {
        let (manifest_version, sage_version) = SageAppPackageManifestParts::v0_defaults();

        SageAppPackageManifest::try_from(SageAppPackageManifestParts {
            manifest_version,
            name: "Test App".into(),
            icon: None,
            sage_version,
            version: "1.0.0".into(),
            permissions: SageRequestedPermissions::new(
                SageRequestedNetworkPermissions::new(
                    [SageNetworkWhitelistEntry::new_unchecked(
                        "https",
                        "api.example.com",
                    )],
                    [],
                    [],
                )
                    .unwrap(),
                SageRequestedCapabilities::new(
                    [UserBridgeCapability::StoragePersistentWebview],
                    [UserBridgeCapability::WalletSendXch],
                ),
            )
                .unwrap(),
            files: vec![SageAppManifestFile::new(
                "index.html".to_string(),
                "a".repeat(64),
                123,
            )
                .unwrap()],
            entry: Some("index.html".into()),
            author: None,
            donation: None,
        })
            .unwrap()
    }

    #[test]
    fn fresh_origin_id_uses_uuid_host_prefix_and_app_id_suffix() {
        let origin = fresh_origin_id("url-abc123");

        assert!(origin.ends_with(".url-abc123"));
        assert_ne!(origin, "url-abc123");

        let prefix = origin.strip_suffix(".url-abc123").unwrap();
        Uuid::parse_str(prefix).unwrap();
    }

    #[tokio::test]
    async fn materialize_installed_app_builds_installed_app() {
        let dir = tempdir().unwrap();
        let app_id = "fake-app".to_string();
        let app_dir = apps_root(dir.path()).join(&app_id);
        let origin_id = fresh_origin_id(&app_id);

        let granted = SageGrantedPermissionsInput::new(
            [UserBridgeCapability::StoragePersistentWebview],
            [SageNetworkWhitelistEntry::new_unchecked(
                "https",
                "api.example.com",
            )],
            BTreeMap::new(),
        );

        let source = FakeInstallSource {
            manifest: sample_manifest(),
            app_id: app_id.clone(),
            source: UserSageAppSource::Zip,
        };

        let prepared_artifact = source.prepare().await.unwrap();

        let installed = materialize_installed_app(
            source,
            prepared_artifact,
            ResolvedInstallTarget {
                app_id: app_id.clone(),
                app_dir,
                origin_id: origin_id.clone(),
                storage: SageAppStorage::Unmanaged,
            },
            granted,
            SageAppWalletScope::AllWallets,
        )
            .await
            .unwrap();

        let common = installed.common();

        assert_eq!(common.id(), app_id);
        assert_eq!(common.origin_id(), origin_id);
        assert_eq!(common.name(), "Test App");
        assert_eq!(common.entry_file(), "index.html");
        assert_eq!(common.icon_file(), None);
        assert_eq!(common.storage(), &SageAppStorage::Unmanaged);
        assert_eq!(installed.source(), &UserSageAppSource::Zip);
    }

    #[test]
    fn create_app_dir_rejects_existing_directory() {
        let dir = tempdir().unwrap();
        let app_dir = dir.path().join("fake-app");

        fs::create_dir_all(&app_dir).unwrap();

        let err = create_app_dir(&app_dir).unwrap_err();

        assert!(err.to_string().contains("app directory already exists"));
    }
}

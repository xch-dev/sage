pub mod types;

use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::lifecycle::types::PersistedUserSageApp;
use crate::system_apps::list_builtin_system_apps;
use crate::types::{
    CorruptedInstalledSageApp, ListedSageApp, PendingStorageCleanupEntry, RetiredAppOriginEntry,
    SageApp, SageNetworkWhitelistEntry, UserSageApp,
};
use anyhow::{Context, Result as AnyResult};

const INSTALLED_METADATA_FILE: &str = ".sage-installed.json";
const PENDING_STORAGE_CLEANUP_FILE: &str = ".sage-pending-storage-cleanup.json";
const RETIRED_APP_ORIGINS_FILE: &str = ".sage-retired-app-origins.json";

pub fn apps_root(base_path: &Path) -> PathBuf {
    base_path.join("apps")
}

pub fn app_dir(base_path: &Path, app_id: &str) -> PathBuf {
    apps_root(base_path).join(app_id)
}

pub fn installed_metadata_path(app_dir: &Path) -> PathBuf {
    app_dir.join(INSTALLED_METADATA_FILE)
}

pub fn pending_storage_cleanup_path(base_path: &Path) -> PathBuf {
    apps_root(base_path).join(PENDING_STORAGE_CLEANUP_FILE)
}

pub fn retired_app_origins_path(base_path: &Path) -> PathBuf {
    apps_root(base_path).join(RETIRED_APP_ORIGINS_FILE)
}

pub fn parse_network_permission_target(value: &str) -> Result<SageNetworkWhitelistEntry, String> {
    let value = value.trim().to_ascii_lowercase();

    let (scheme, host) = value
        .split_once("://")
        .ok_or_else(|| format!("invalid network entry (missing scheme): {value}"))?;

    SageNetworkWhitelistEntry::new(scheme, host).map_err(|err| err.to_string())
}

pub fn read_installed_user_app_from_dir(dir: &Path) -> AnyResult<UserSageApp> {
    let path = installed_metadata_path(dir);
    let text =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    let persisted: PersistedUserSageApp =
        serde_json::from_str(&text).context("failed to parse installed app metadata")?;
    persisted.try_into()
}

pub fn write_installed_app_metadata(app: &UserSageApp, app_dir: &Path) -> AnyResult<()> {
    let path = installed_metadata_path(app_dir);
    let persisted: PersistedUserSageApp = app.into();
    let text = serde_json::to_string_pretty(&persisted)
        .map_err(|err| anyhow::anyhow!("failed to serialize installed app metadata: {err}"))?;
    fs::write(path, format!("{text}\n"))?;
    Ok(())
}

pub fn read_installed_app_by_id(base_path: &Path, app_id: &str) -> AnyResult<UserSageApp> {
    let dir = app_dir(base_path, app_id);
    read_installed_user_app_from_dir(&dir)
}

pub fn list_installed_apps_internal(root: &Path) -> AnyResult<Vec<ListedSageApp>> {
    if !root.exists() {
        return Ok(Vec::new());
    }

    let mut apps = Vec::new();

    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();

        if !entry.file_type()?.is_dir() {
            continue;
        }

        if path
            .file_name()
            .and_then(|s| s.to_str())
            .is_some_and(|s| s.starts_with(".tmp-"))
        {
            continue;
        }

        let Some(id) = path
            .file_name()
            .and_then(|s| s.to_str())
            .map(std::string::ToString::to_string)
        else {
            continue;
        };

        let metadata_path = installed_metadata_path(&path);
        if !metadata_path.is_file() {
            continue;
        }

        match read_installed_user_app_from_dir(&path) {
            Ok(app) => apps.push(ListedSageApp::User(app)),
            Err(err) => apps.push(ListedSageApp::Corrupted(CorruptedInstalledSageApp {
                id,
                app_dir: path.to_string_lossy().to_string(),
                error: err.to_string(),
            })),
        }
    }
    for app in list_builtin_system_apps()? {
        if let SageApp::System(app) = app {
            apps.push(ListedSageApp::System(app));
        }
    }

    apps.sort_by(|a, b| {
        let a_key = match a {
            ListedSageApp::User(app) => app.common.name.to_lowercase(),
            ListedSageApp::System(app) => app.common.name.to_lowercase(),
            ListedSageApp::Corrupted(app) => app.id.to_lowercase(),
        };

        let b_key = match b {
            ListedSageApp::User(app) => app.common.name.to_lowercase(),
            ListedSageApp::System(app) => app.common.name.to_lowercase(),
            ListedSageApp::Corrupted(app) => app.id.to_lowercase(),
        };

        a_key.cmp(&b_key)
    });

    Ok(apps)
}

pub fn read_pending_storage_cleanup_entries(
    base_path: &Path,
) -> AnyResult<Vec<PendingStorageCleanupEntry>> {
    let path = pending_storage_cleanup_path(base_path);

    if !path.exists() {
        return Ok(Vec::new());
    }

    let text =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;

    let entries = serde_json::from_str::<Vec<PendingStorageCleanupEntry>>(&text)
        .with_context(|| format!("failed to parse {}", path.display()))?;

    Ok(entries)
}

pub fn write_pending_storage_cleanup_entries(
    base_path: &Path,
    entries: &[PendingStorageCleanupEntry],
) -> AnyResult<()> {
    let root = apps_root(base_path);
    fs::create_dir_all(&root)
        .with_context(|| format!("failed to create apps root {}", root.display()))?;

    let path = pending_storage_cleanup_path(base_path);
    let text = serde_json::to_string_pretty(entries).map_err(|err| {
        anyhow::anyhow!("failed to serialize pending storage cleanup entries: {err}")
    })?;
    fs::write(&path, format!("{text}\n"))
        .with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

pub fn read_retired_app_origins(base_path: &Path) -> AnyResult<Vec<RetiredAppOriginEntry>> {
    let path = retired_app_origins_path(base_path);

    if !path.exists() {
        return Ok(Vec::new());
    }

    let text =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;

    let entries = serde_json::from_str::<Vec<RetiredAppOriginEntry>>(&text)
        .with_context(|| format!("failed to parse {}", path.display()))?;

    Ok(entries)
}

pub fn write_retired_app_origins(
    base_path: &Path,
    entries: &[RetiredAppOriginEntry],
) -> AnyResult<()> {
    let root = apps_root(base_path);
    fs::create_dir_all(&root)
        .with_context(|| format!("failed to create apps root {}", root.display()))?;

    let path = retired_app_origins_path(base_path);
    let text = serde_json::to_string_pretty(entries)
        .map_err(|err| anyhow::anyhow!("failed to serialize retired app origins: {err}"))?;

    fs::write(&path, format!("{text}\n"))
        .with_context(|| format!("failed to write {}", path.display()))?;

    Ok(())
}

pub fn read_installed_user_app_by_origin_id(
    base_path: &Path,
    origin_id: &str,
) -> AnyResult<UserSageApp> {
    let root = apps_root(base_path);

    for entry in list_installed_apps_internal(&root)? {
        if let ListedSageApp::User(app) = entry
            && app.common.origin_id == origin_id
        {
            return Ok(app);
        }
    }

    Err(anyhow::anyhow!(
        "no installed app found for origin id {origin_id}"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    use crate::types::{
        InstalledSageAppStorage, SageAppCommon, SageAppManifestFile, SageAppPackageManifest,
        SageAppPackageManifestParts, SageAppSnapshot, SageGrantedPermissions,
        SageRequestedPermissions, UserSageApp, UserSageAppSource,
    };

    fn sample_app(base: &Path, app_id: &str, origin_id: &str) -> UserSageApp {
        let dir = app_dir(base, app_id);
        fs::create_dir_all(&dir).unwrap();

        let manifest = SageAppPackageManifest::try_from(SageAppPackageManifestParts {
            name: "Test App".into(),
            version: "1.0.0".into(),
            permissions: SageRequestedPermissions::empty(),
            files: Vec::from([SageAppManifestFile {
                path: "index.html".into(),
                sha256: "a".repeat(64),
                size: 1,
            }]),
            entry: Some("index.html".into()),
            icon: Some("icon.png".into()),
            author: None,
            donation: None,
        })
        .unwrap();

        let granted_permissions =
            SageGrantedPermissions::new(manifest.permissions(), [], []).unwrap();

        let snapshot = SageAppSnapshot {
            manifest_hash: "hash".into(),
            snapshot_dir: dir.to_string_lossy().to_string(),
            total_bytes: 1,
            manifest: manifest.clone(),
        };

        let common = SageAppCommon::new(
            app_id.into(),
            origin_id.into(),
            dir.to_string_lossy().to_string(),
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
    fn installed_app_metadata_round_trips_origin_id_and_storage() {
        let tmp = tempdir().unwrap();
        let mut app = sample_app(tmp.path(), "url-abc123", "origin-1");
        app.common.storage = InstalledSageAppStorage::Unmanaged;

        let dir = app_dir(tmp.path(), &app.common.id);
        write_installed_app_metadata(&app, &dir).unwrap();

        let read_back = read_installed_app_by_id(tmp.path(), &app.common.id).unwrap();
        assert_eq!(read_back.common.id, app.common.id);
        assert_eq!(read_back.common.origin_id, app.common.origin_id);
        assert_eq!(read_back.common.storage, app.common.storage);
    }

    #[test]
    fn read_installed_app_by_origin_id_finds_matching_app() {
        let dir = tempdir().unwrap();

        let app_a = sample_app(dir.path(), "app-a", "origin-a");
        let app_b = sample_app(dir.path(), "app-b", "origin-b");

        write_installed_app_metadata(&app_a, Path::new(&app_a.common.app_dir)).unwrap();
        write_installed_app_metadata(&app_b, Path::new(&app_b.common.app_dir)).unwrap();

        let found = read_installed_user_app_by_origin_id(dir.path(), "origin-b").unwrap();
        assert_eq!(found.common.id, "app-b");
    }

    #[test]
    fn read_installed_app_by_origin_id_errors_when_missing() {
        let dir = tempdir().unwrap();
        let err = read_installed_user_app_by_origin_id(dir.path(), "missing").unwrap_err();
        assert!(
            err.to_string()
                .contains("no installed app found for origin id")
        );
    }
}

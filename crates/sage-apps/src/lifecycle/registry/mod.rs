pub mod types;

use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result as AnyResult};

use crate::lifecycle::types::PersistedUserSageApp;
use crate::system_apps::list_builtin_system_apps;
use crate::types::{
    CorruptedInstalledSageApp, ListedSageApp, PendingStorageCleanupEntry, RetiredAppOriginEntry,
    SageApp, SageNetworkWhitelistEntry, UserSageApp,
};

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

    let persisted: PersistedUserSageApp = serde_json::from_str(&text)
        .with_context(|| format!("failed to parse installed app metadata {}", path.display()))?;

    persisted.try_into()
}

pub fn write_installed_app_metadata(app: &UserSageApp, app_dir: &Path) -> AnyResult<()> {
    let path = installed_metadata_path(app_dir);
    let persisted = PersistedUserSageApp::from(app);

    let text = serde_json::to_string_pretty(&persisted)
        .map_err(|err| anyhow::anyhow!("failed to serialize installed app metadata: {err}"))?;

    fs::write(&path, format!("{text}\n"))
        .with_context(|| format!("failed to write {}", path.display()))?;

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
            .map(str::to_string)
        else {
            continue;
        };

        let metadata_path = installed_metadata_path(&path);
        if !metadata_path.is_file() {
            continue;
        }

        match read_installed_user_app_from_dir(&path) {
            Ok(app) => apps.push(ListedSageApp::User(app)),
            Err(err) => apps.push(ListedSageApp::Corrupted(CorruptedInstalledSageApp::new(
                id,
                path.to_string_lossy().to_string(),
                err.to_string(),
            ))),
        }
    }

    for app in list_builtin_system_apps()? {
        if let SageApp::System(app) = app {
            apps.push(ListedSageApp::System(app));
        }
    }

    apps.sort_by_key(listed_app_sort_key);

    Ok(apps)
}

fn listed_app_sort_key(app: &ListedSageApp) -> String {
    match app {
        ListedSageApp::User(app) => app.common().name().to_lowercase(),
        ListedSageApp::System(app) => app.common().name().to_lowercase(),
        ListedSageApp::Corrupted(app) => app.id().to_lowercase(),
    }
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
            && app.common().origin_id() == origin_id
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

    use crate::lifecycle::storage::record_storage_cleanup_failure;
    use crate::types::{
        InstalledSageAppStorage, ListedSageApp, PendingStorageCleanupTarget, RetiredAppOriginEntry,
        SageAppCommon, SageAppManifestFile, SageAppPackageManifest, SageAppPackageManifestParts,
        SageAppSnapshot, SageGrantedPermissions, SageNetworkWhitelistEntry,
        SageRequestedPermissions, UserSageApp, UserSageAppSource,
    };
    use std::fs;
    use tempfile::tempdir;

    fn write_index(dir: &Path) {
        fs::create_dir_all(dir).unwrap();
        fs::write(dir.join("index.html"), "x").unwrap();
    }

    fn sample_manifest_file(path: &str, size: u64) -> SageAppManifestFile {
        SageAppManifestFile::new(path, "a".repeat(64), size).unwrap()
    }

    fn sample_manifest(name: &str) -> SageAppPackageManifest {
        SageAppPackageManifest::try_from(SageAppPackageManifestParts {
            name: name.to_string(),
            version: "1.0.0".to_string(),
            permissions: SageRequestedPermissions::empty(),
            files: vec![sample_manifest_file("index.html", 1)],
            entry: Some("index.html".to_string()),
            icon: None,
            author: None,
            donation: None,
        })
        .unwrap()
    }

    fn sample_app(base: &Path, app_id: &str, origin_id: &str) -> UserSageApp {
        sample_app_named(base, app_id, origin_id, "Test App")
    }

    fn sample_app_named(base: &Path, app_id: &str, origin_id: &str, name: &str) -> UserSageApp {
        let dir = app_dir(base, app_id);
        write_index(&dir);

        let manifest = sample_manifest(name);
        let granted_permissions =
            SageGrantedPermissions::new(manifest.permissions(), [], []).unwrap();

        let snapshot =
            SageAppSnapshot::new("hash", dir.to_string_lossy().to_string(), manifest).unwrap();

        let common = SageAppCommon::new(
            app_id,
            origin_id,
            dir.to_string_lossy().to_string(),
            granted_permissions,
            InstalledSageAppStorage::Unmanaged,
            snapshot,
        )
        .unwrap();

        UserSageApp::new_installed(
            common,
            UserSageAppSource::Url {
                app_url: "https://example.com/app/".into(),
                manifest_url: "https://example.com/app/sage-manifest.json".into(),
            },
        )
    }

    fn without_system_apps(listed: Vec<ListedSageApp>) -> Vec<ListedSageApp> {
        listed
            .into_iter()
            .filter(|entry| !matches!(entry, ListedSageApp::System(_)))
            .collect()
    }

    #[test]
    fn installed_app_metadata_round_trips_origin_id_and_storage() {
        let tmp = tempdir().unwrap();
        let app = sample_app(tmp.path(), "url-abc123", "origin-1");

        let dir = app_dir(tmp.path(), app.common().id());
        write_installed_app_metadata(&app, &dir).unwrap();

        let read_back = read_installed_app_by_id(tmp.path(), app.common().id()).unwrap();

        assert_eq!(read_back.common().id(), app.common().id());
        assert_eq!(read_back.common().origin_id(), app.common().origin_id());
        assert_eq!(read_back.common().storage(), app.common().storage());
        assert_eq!(
            read_back.common().granted_permissions(),
            app.common().granted_permissions()
        );
        assert!(!read_back.common().capability_flags().has_external_access());
    }

    #[test]
    fn corrupted_metadata_is_reported_as_corrupted_listing() {
        let base = tempdir().unwrap();
        let dir = app_dir(base.path(), "broken-app");
        fs::create_dir_all(&dir).unwrap();

        fs::write(dir.join(".sage-installed.json"), "{ definitely not json").unwrap();

        let listed =
            without_system_apps(list_installed_apps_internal(&apps_root(base.path())).unwrap());
        assert_eq!(listed.len(), 1);

        match &listed[0] {
            ListedSageApp::Corrupted(app) => {
                assert_eq!(app.id(), "broken-app");
                assert!(!app.error().is_empty());
            }
            ListedSageApp::User(_) | ListedSageApp::System(_) => {
                panic!("expected corrupted app listing");
            }
        }
    }

    #[test]
    fn corrupted_persisted_network_entry_is_reported_as_corrupted_listing() {
        let base = tempdir().unwrap();
        let dir = app_dir(base.path(), "broken-app");
        fs::create_dir_all(&dir).unwrap();
        let snapshot_dir = dir.to_string_lossy();

        fs::write(
            dir.join(".sage-installed.json"),
            format!(
                r#"{{
  "id": "broken-app",
  "originId": "broken-app",
  "appDir": "{snapshot_dir}",
  "grantedPermissions": {{
    "capabilities": [],
    "network": {{
      "whitelist": []
    }}
  }},
  "storage": {{
    "kind": "unmanaged"
  }},
  "source": {{
    "kind": "zip"
  }},
  "activeSnapshot": {{
    "manifestHash": "hash",
    "snapshotDir": "{snapshot_dir}",
    "manifest": {{
      "name": "Broken App",
      "version": "1.0.0",
      "permissions": {{
        "network": {{
          "whitelist": {{
            "required": ["https://ok.example.com/path"],
            "optional": []
          }}
        }},
        "capabilities": {{
          "required": [],
          "optional": []
        }}
      }},
      "files": [
        {{
          "path": "index.html",
          "sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
          "size": 1
        }}
      ],
      "entry": "index.html",
      "icon": null
    }}
  }}
}}"#
            ),
        )
        .unwrap();

        let listed =
            without_system_apps(list_installed_apps_internal(&apps_root(base.path())).unwrap());
        assert_eq!(listed.len(), 1);

        match &listed[0] {
            ListedSageApp::Corrupted(app) => {
                assert!(
                    app.error().contains("network entry")
                        || app.error().contains("invalid host")
                        || app
                            .error()
                            .contains("failed to parse installed app metadata"),
                    "unexpected error: {}",
                    app.error()
                );
            }
            ListedSageApp::User(_) | ListedSageApp::System(_) => {
                panic!("expected corrupted app listing");
            }
        }
    }

    #[test]
    fn installed_apps_are_sorted_by_name() {
        let base = tempdir().unwrap();

        let alpha = sample_app_named(base.path(), "a", "a", "Alpha");
        write_installed_app_metadata(&alpha, Path::new(alpha.common().app_dir())).unwrap();

        let zeta = sample_app_named(base.path(), "z", "z", "Zeta");
        write_installed_app_metadata(&zeta, Path::new(zeta.common().app_dir())).unwrap();

        let listed =
            without_system_apps(list_installed_apps_internal(&apps_root(base.path())).unwrap());

        let names: Vec<_> = listed
            .into_iter()
            .map(|entry| match entry {
                ListedSageApp::User(app) => app.common().name().to_string(),
                ListedSageApp::System(app) => app.common().name().to_string(),
                ListedSageApp::Corrupted(app) => app.id().to_string(),
            })
            .collect();

        assert_eq!(names, vec!["Alpha".to_string(), "Zeta".to_string()]);
    }

    #[test]
    fn list_installed_apps_ignores_tmp_directories() {
        let base = tempdir().unwrap();
        let root = apps_root(base.path());
        fs::create_dir_all(root.join(".tmp-123")).unwrap();

        let listed =
            without_system_apps(list_installed_apps_internal(&apps_root(base.path())).unwrap());
        assert!(listed.is_empty());
    }

    #[test]
    fn list_installed_apps_ignores_directories_without_metadata() {
        let base = tempdir().unwrap();
        let root = apps_root(base.path());
        fs::create_dir_all(root.join("missing-metadata")).unwrap();

        let listed =
            without_system_apps(list_installed_apps_internal(&apps_root(base.path())).unwrap());
        assert!(listed.is_empty());
    }

    #[test]
    fn parse_network_permission_target_normalizes_case() {
        let parsed = parse_network_permission_target("HTTPS://Example.COM").unwrap();
        assert_eq!(
            parsed,
            SageNetworkWhitelistEntry::new("https", "example.com").unwrap()
        );
    }

    #[test]
    fn parse_network_permission_target_rejects_missing_scheme_separator() {
        let err = parse_network_permission_target("example.com").unwrap_err();
        assert!(err.contains("missing scheme"));
    }

    #[test]
    fn parse_network_permission_target_rejects_unsupported_scheme() {
        let err = parse_network_permission_target("http://example.com").unwrap_err();
        assert!(err.contains("only https and wss allowed"));
    }

    #[test]
    fn parse_network_permission_target_rejects_invalid_host_chars() {
        assert!(parse_network_permission_target("https://example.com/path").is_err());
        assert!(parse_network_permission_target("https://example.com?x=1").is_err());
        assert!(parse_network_permission_target("https://example.com#frag").is_err());
        assert!(parse_network_permission_target("https://exa mple.com").is_err());
    }

    #[test]
    fn read_installed_app_by_origin_id_finds_matching_app() {
        let dir = tempdir().unwrap();

        let app_a = sample_app(dir.path(), "app-a", "origin-a");
        let app_b = sample_app(dir.path(), "app-b", "origin-b");

        write_installed_app_metadata(&app_a, Path::new(app_a.common().app_dir())).unwrap();
        write_installed_app_metadata(&app_b, Path::new(app_b.common().app_dir())).unwrap();

        let found = read_installed_user_app_by_origin_id(dir.path(), "origin-b").unwrap();
        assert_eq!(found.common().id(), "app-b");
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

    #[test]
    fn pending_storage_cleanup_entries_round_trip() {
        let base = tempdir().unwrap();

        let app = sample_app(base.path(), "app-1", "origin-1");
        record_storage_cleanup_failure(base.path(), &app, "boom").unwrap();

        let entries = read_pending_storage_cleanup_entries(base.path()).unwrap();
        write_pending_storage_cleanup_entries(base.path(), &entries).unwrap();

        let loaded = read_pending_storage_cleanup_entries(base.path()).unwrap();
        assert_eq!(loaded, entries);
        assert_eq!(loaded[0].target(), &PendingStorageCleanupTarget::Unmanaged);
    }

    #[test]
    fn retired_app_origins_round_trip() {
        let base = tempdir().unwrap();

        let app = sample_app(base.path(), "app-1", "origin-1");

        let entries = vec![RetiredAppOriginEntry::new(&app, true)];
        write_retired_app_origins(base.path(), &entries).unwrap();

        let loaded = read_retired_app_origins(base.path()).unwrap();
        assert_eq!(loaded, entries);
    }
}

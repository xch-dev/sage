use std::{fs, path::PathBuf};

use anyhow::{Context, Result as AnyResult, anyhow};
use sha2::{Digest, Sha256};
use tauri::command;

use crate::host::Result;
use crate::lifecycle::{manifest_entry_file, manifest_icon_file};
use crate::lifecycle::flags::get_app_flags;
use crate::permissions::{normalize_and_validate_requested_permissions};
use crate::permissions::capabilities::definitions::{get_user_capability_definition};
use crate::permissions::capabilities::get_effective_granted_capabilities;
use crate::permissions::capabilities::validation::validate_granted_capabilities;
use crate::types::{InstalledSageAppStorage, SageApp, SageAppCommon, SageAppSnapshot, SageAppPackageManifest, SageGrantedNetworkPermissions, SageGrantedPermissions, UserSageAppSource, UserSageApp};

macro_rules! sandbox_test_id_prefix {
    () => {
        "__sage_test_"
    };
}

pub const SANDBOX_TEST_ID_PREFIX: &str = sandbox_test_id_prefix!();

pub const BUILTIN_STORAGE_ISOLATION_PERSISTENT_ID: &str =
    concat!(sandbox_test_id_prefix!(), "storage_isolation_persistent");
pub const BUILTIN_STORAGE_ISOLATION_INCOGNITO_ID: &str =
    concat!(sandbox_test_id_prefix!(), "storage_isolation_incognito");
pub const BUILTIN_PERSISTENCE_PERSISTENT_ID: &str =
    concat!(sandbox_test_id_prefix!(), "persistence_persistent");
pub const BUILTIN_PERSISTENCE_INCOGNITO_ID: &str =
    concat!(sandbox_test_id_prefix!(), "persistence_incognito");
pub const BUILTIN_STORAGE_CLEAR_PERSISTENT_ID: &str =
    concat!(sandbox_test_id_prefix!(), "storage_clear_persistent");
pub const BUILTIN_NETWORK_ALLOW_A_ID: &str =
    concat!(sandbox_test_id_prefix!(), "network_allow_a");
pub const BUILTIN_NETWORK_ALLOW_B_ID: &str =
    concat!(sandbox_test_id_prefix!(), "network_allow_b");

#[derive(Debug, Clone, Copy)]
pub struct BuiltinTestAppSpec {
    pub app_id: &'static str,
    pub dir_name: &'static str,
}

const BUILTIN_TEST_APPS: &[BuiltinTestAppSpec] = &[
    BuiltinTestAppSpec {
        app_id: BUILTIN_STORAGE_ISOLATION_PERSISTENT_ID,
        dir_name: "sage-storage-isolation-persistent",
    },
    BuiltinTestAppSpec {
        app_id: BUILTIN_STORAGE_ISOLATION_INCOGNITO_ID,
        dir_name: "sage-storage-isolation-incognito",
    },
    BuiltinTestAppSpec {
        app_id: BUILTIN_PERSISTENCE_PERSISTENT_ID,
        dir_name: "storage-persistence-persistent",
    },
    BuiltinTestAppSpec {
        app_id: BUILTIN_PERSISTENCE_INCOGNITO_ID,
        dir_name: "storage-persistence-incognito",
    },
    BuiltinTestAppSpec {
        app_id: BUILTIN_STORAGE_CLEAR_PERSISTENT_ID,
        dir_name: "storage-clear-persistent",
    },
    BuiltinTestAppSpec {
        app_id: BUILTIN_NETWORK_ALLOW_A_ID,
        dir_name: "network-allow-a",
    },
    BuiltinTestAppSpec {
        app_id: BUILTIN_NETWORK_ALLOW_B_ID,
        dir_name: "network-allow-b",
    },
];

pub fn builtin_test_app_spec(app_id: &str) -> Option<&'static BuiltinTestAppSpec> {
    BUILTIN_TEST_APPS.iter().find(|spec| spec.app_id == app_id)
}

pub fn builtin_apps_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("builtin-apps")
        .join("dist")
}

pub fn builtin_test_apps_root() -> PathBuf {
    builtin_apps_root().join("test-apps")
}

pub fn builtin_runtime_apps_root() -> PathBuf {
    builtin_apps_root().join("runtime-apps")
}

pub fn builtin_test_app_dir(app_id: &str) -> AnyResult<Option<PathBuf>> {
    let Some(spec) = builtin_test_app_spec(app_id) else {
        return Ok(None);
    };

    Ok(Some(builtin_test_apps_root().join(spec.dir_name)))
}

#[cfg(any(target_os = "macos", target_os = "ios"))]
fn builtin_storage(app_id: &str) -> InstalledSageAppStorage {
    let mut hasher = Sha256::new();
    hasher.update(format!("builtin-storage:{app_id}").as_bytes());
    let digest = hasher.finalize();

    InstalledSageAppStorage::AppleDataStore {
        identifier_hex: hex::encode(&digest[..16]),
    }
}

#[cfg(target_os = "windows")]
fn builtin_storage(app_id: &str) -> InstalledSageAppStorage {
    InstalledSageAppStorage::WindowsProfile {
        directory_name: format!("builtin-profile-{app_id}"),
    }
}

#[cfg(not(any(target_os = "macos", target_os = "ios", target_os = "windows")))]
fn builtin_storage(_app_id: &str) -> InstalledSageAppStorage {
    InstalledSageAppStorage::Unmanaged
}

fn read_builtin_manifest(app_dir: &PathBuf) -> AnyResult<SageAppPackageManifest> {
    let manifest_path = app_dir.join("sage-manifest.json");

    let manifest_text = fs::read_to_string(&manifest_path).with_context(|| {
        format!(
            "failed to read builtin test app manifest {}",
            manifest_path.display()
        )
    })?;

    let manifest: SageAppPackageManifest =
        serde_json::from_str(&manifest_text).with_context(|| {
            format!(
                "failed to parse builtin test app manifest {}",
                manifest_path.display()
            )
        })?;

    Ok(manifest)
}

fn compute_total_bytes(app_dir: &PathBuf) -> AnyResult<u64> {
    let mut total_bytes = 0_u64;

    for entry in fs::read_dir(app_dir)
        .with_context(|| format!("failed to read builtin test app dir {}", app_dir.display()))?
    {
        let entry = entry.with_context(|| {
            format!(
                "failed to read entry in builtin test app dir {}",
                app_dir.display()
            )
        })?;

        let metadata = entry.metadata().with_context(|| {
            format!(
                "failed to read metadata for builtin test app file {}",
                entry.path().display()
            )
        })?;

        if metadata.is_file() {
            total_bytes = total_bytes
                .checked_add(metadata.len())
                .ok_or_else(|| anyhow!("builtin test app total size overflow"))?;
        }
    }

    Ok(total_bytes)
}

pub fn build_builtin_test_app(app_id: &str) -> AnyResult<Option<SageApp>> {
    let Some(spec) = builtin_test_app_spec(app_id) else {
        return Ok(None);
    };

    let app_dir = builtin_test_apps_root().join(spec.dir_name);

    if !app_dir.is_dir() {
        return Err(anyhow!(
            "builtin test app directory does not exist: {}",
            app_dir.display()
        ));
    }

    let mut manifest = read_builtin_manifest(&app_dir)?;
    manifest.permissions =
        normalize_and_validate_requested_permissions(&manifest.permissions)?;

    let mut user_granted_capabilities = Vec::new();

    for capability in manifest
        .permissions
        .capabilities
        .required
        .iter()
        .chain(manifest.permissions.capabilities.optional.iter())
    {
        let definition = get_user_capability_definition(*capability);

        if definition.flags.user_grantable {
            user_granted_capabilities.push(*capability);
        }
    }

    user_granted_capabilities.sort();
    user_granted_capabilities.dedup();

    let granted_permissions = SageGrantedPermissions {
        capabilities: user_granted_capabilities,
        network: SageGrantedNetworkPermissions {
            whitelist: manifest.permissions.network.whitelist.required.clone(),
        },
    };

    validate_granted_capabilities(
        &manifest.permissions,
        &granted_permissions.capabilities,
    )?;
    let effective_capabilities = get_effective_granted_capabilities(
        &manifest.permissions,
        &granted_permissions.capabilities,
    )?;

    let capability_flags = get_app_flags(&effective_capabilities, None)?;

    let entry_file_name = manifest_entry_file(&manifest).to_string();
    let icon_file_name = manifest_icon_file(&manifest).to_string();

    let entry_file = app_dir.join(&entry_file_name);
    if !entry_file.is_file() {
        return Err(anyhow!(
            "builtin test app entry file does not exist: {}",
            entry_file.display()
        ));
    }

    let icon_file = app_dir.join(&icon_file_name);
    if !icon_file.is_file() {
        return Err(anyhow!(
            "builtin test app icon file does not exist: {}",
            icon_file.display()
        ));
    }

    let total_bytes = compute_total_bytes(&app_dir)?;

    let app = UserSageApp {
        common: SageAppCommon {
            id: spec.app_id.to_string(),
            origin_id: spec.app_id.to_string(),
            name: manifest.name.clone(),
            version: manifest.version.clone(),
            app_dir: app_dir.to_string_lossy().to_string(),
            entry_file: entry_file_name,
            icon_file: icon_file_name,
            requested_permissions: manifest.permissions.clone(),
            granted_permissions,
            capability_flags,
            storage: builtin_storage(spec.app_id),
            active_snapshot: SageAppSnapshot {
                manifest_hash: format!("builtin:{}", spec.app_id),
                snapshot_dir: app_dir.to_string_lossy().to_string(),
                total_bytes,
                manifest,
            },
        },
        source: UserSageAppSource::Zip,
        pending_update: None,
    };

    Ok(Some(SageApp::User(app)))
}

#[command]
#[specta::specta]
pub async fn get_builtin_test_app(
    app_id: String,
) -> Result<Option<SageApp>> {
    build_builtin_test_app(&app_id).map_err(|err| {
        std::io::Error::other(format!("failed to load builtin test app: {err}")).into()
    })
}

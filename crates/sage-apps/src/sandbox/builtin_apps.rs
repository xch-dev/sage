use anyhow::{Context, Result as AnyResult};
use std::path::Path;
use std::{fs, path::PathBuf};
use sha2::{Digest, Sha256};

use crate::system_apps::AppBuildError;
use crate::types::{
    InstalledSageAppStorage, SageApp, SageAppCommon, SageAppIdentity, SageAppPackageManifest,
    SageAppSnapshot, SageGrantedPermissions, UserSageApp, UserSageAppSource,
};
use crate::utils::builtin_apps_root;

macro_rules! sandbox_test_id_prefix {
    () => {
        "__sage_test_"
    };
}

macro_rules! runtime_id_prefix {
    () => {
        "__sage_runtime_"
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
pub const BUILTIN_NETWORK_ALLOW_A_ID: &str = concat!(sandbox_test_id_prefix!(), "network_allow_a");
pub const BUILTIN_NETWORK_ALLOW_B_ID: &str = concat!(sandbox_test_id_prefix!(), "network_allow_b");

pub const BUILTIN_STORAGE_CLEAR_PROBE_RUNTIME_ID: &str =
    concat!(runtime_id_prefix!(), "storage_clear_probe");

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

pub fn builtin_test_apps_root() -> PathBuf {
    builtin_apps_root().join("sandbox-test")
}

pub fn builtin_runtime_apps_root() -> PathBuf {
    builtin_apps_root().join("runtime")
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

fn read_builtin_manifest(app_dir: &Path) -> AnyResult<SageAppPackageManifest> {
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

pub fn build_builtin_test_app(app_id: &str) -> Result<Option<SageApp>, AppBuildError> {
    let Some(spec) = builtin_test_app_spec(app_id) else {
        return Ok(None);
    };

    let app_dir = builtin_test_apps_root().join(spec.dir_name);

    if !app_dir.is_dir() {
        return Err(AppBuildError::AppDirMissing);
    }

    let manifest = read_builtin_manifest(&app_dir).map_err(|_| AppBuildError::ManifestFailure)?;

    let granted_permissions = SageGrantedPermissions::new(
        manifest.permissions(),
        manifest.permissions().capabilities().user_grantable(),
        manifest
            .permissions()
            .network()
            .whitelist()
            .required()
            .cloned(),
    ).map_err(|_| AppBuildError::InternalError)?;

    let snapshot = SageAppSnapshot::new(
        format!("builtin:{}", spec.app_id),
        app_dir.to_string_lossy().to_string(),
        manifest.clone(),
    ).map_err(|_| AppBuildError::InternalError)?;

    let common = SageAppCommon::new(
        SageAppIdentity::new(
            spec.app_id.to_string(),
            spec.app_id.to_string(),
            app_dir.to_string_lossy().to_string(),
        ).map_err(|_| AppBuildError::InternalError)?,
        granted_permissions,
        builtin_storage(spec.app_id),
        snapshot,
    ).map_err(|_| AppBuildError::InternalError)?;



    let entry_file = app_dir.join(common.entry_file());
    if !entry_file.is_file() {
        return Err(AppBuildError::EntryFileNotFound);
    }

    let app = UserSageApp::new_installed(common, UserSageAppSource::Zip);

    Ok(Some(SageApp::User(app)))
}

pub fn build_builtin_runtime_app(
    app_id: &str,
) -> Result<Option<SageApp>, AppBuildError> {
    if app_id != BUILTIN_STORAGE_CLEAR_PROBE_RUNTIME_ID {
        return Ok(None);
    }

    let app_dir = builtin_runtime_apps_root().join("storage-clear-probe");

    if !app_dir.is_dir() {
        return Err(AppBuildError::AppDirMissing);
    }

    let manifest = read_builtin_manifest(&app_dir).map_err(|_| AppBuildError::ManifestFailure)?;

    let granted_permissions = SageGrantedPermissions::for_builtin_requested(
        manifest.permissions(),
    )
        .map_err(|err| {
            eprintln!("runtime app granted_permissions failed: {err}");
            AppBuildError::InternalError
        })?;

    let snapshot = SageAppSnapshot::new(
        format!("builtin-runtime:{app_id}"),
        app_dir.to_string_lossy().to_string(),
        manifest.clone(),
    )
        .map_err(|err| {
            eprintln!("runtime app snapshot failed: {err}");
            AppBuildError::InternalError
        })?;

    let common = SageAppCommon::new(
        SageAppIdentity::new(
            app_id,
            app_id,
            app_dir.to_string_lossy().to_string(),
        )
            .map_err(|err| {
                eprintln!("runtime app identity failed: {err}");
                AppBuildError::InternalError
            })?,
        granted_permissions,
        InstalledSageAppStorage::Unmanaged,
        snapshot,
    )
        .map_err(|err| {
            eprintln!("runtime app common failed: {err}");
            AppBuildError::InternalError
        })?;

    let entry_file = app_dir.join(common.entry_file());
    if !entry_file.is_file() {
        return Err(AppBuildError::EntryFileNotFound);
    }

    Ok(Some(SageApp::User(UserSageApp::new_installed(
        common,
        UserSageAppSource::Zip,
    ))))
}

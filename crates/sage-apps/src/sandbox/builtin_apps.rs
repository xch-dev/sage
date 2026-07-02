use std::collections::BTreeMap;
use std::path::Path;
use std::{fs, path::PathBuf};

use anyhow::{Context, Result as AnyResult};
#[cfg(any(target_os = "macos", target_os = "ios"))]
use sha2::{Digest, Sha256};

use crate::{
    AppBuildError, SageApp, SageAppCommon, SageAppIdentity, SageAppPackageManifest,
    SageAppSnapshot, SageAppStorage, SageAppWalletScope, SageGrantedPermissions, UserSageApp,
    UserSageAppSource, builtin_apps_root,
};

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

pub const BUILTIN_ORIGIN_CLEANUP_RUNTIME_ID: &str = concat!(runtime_id_prefix!(), "origin_cleanup");

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

#[cfg(any(target_os = "macos", target_os = "ios"))]
fn builtin_storage(app_id: &str) -> SageAppStorage {
    let mut hasher = Sha256::new();
    hasher.update(format!("builtin-storage:{app_id}").as_bytes());
    let digest = hasher.finalize();

    SageAppStorage::AppleDataStore {
        identifier_hex: hex::encode(&digest[..16]),
    }
}

#[cfg(target_os = "windows")]
fn builtin_storage(app_id: &str) -> SageAppStorage {
    SageAppStorage::WindowsProfile {
        directory_name: format!("builtin-profile-{app_id}"),
    }
}

#[cfg(not(any(target_os = "macos", target_os = "ios", target_os = "windows")))]
fn builtin_storage(_app_id: &str) -> SageAppStorage {
    SageAppStorage::Unmanaged
}

fn read_builtin_manifest(app_dir: &Path) -> AnyResult<SageAppPackageManifest> {
    let manifest_path = app_dir.join("sage-manifest.json");

    let manifest_text = fs::read_to_string(&manifest_path).with_context(|| {
        format!(
            "failed to read builtin app manifest {}",
            manifest_path.display()
        )
    })?;

    serde_json::from_str::<SageAppPackageManifest>(&manifest_text).with_context(|| {
        format!(
            "failed to parse builtin app manifest {}",
            manifest_path.display()
        )
    })
}

pub fn build_builtin_test_app(app_id: &str) -> Result<Option<SageApp>, AppBuildError> {
    let Some(spec) = builtin_test_app_spec(app_id) else {
        return Ok(None);
    };

    let app_dir = builtin_test_apps_root().join(spec.dir_name);

    if !app_dir.is_dir() {
        return Err(AppBuildError::AppDirMissing);
    }

    let manifest = read_builtin_manifest(&app_dir)
        .map_err(|err| AppBuildError::ManifestFailure(format!("{err:#}")))?;

    let granted_permissions = SageGrantedPermissions::new(
        manifest.permissions(),
        manifest.permissions().capabilities().user_grantable(),
        manifest
            .permissions()
            .network()
            .whitelist()
            .required()
            .cloned(),
        BTreeMap::new(),
    )
    .map_err(|_| AppBuildError::InternalError)?;

    let snapshot = SageAppSnapshot::new(
        format!("builtin:{}", spec.app_id),
        app_dir.to_string_lossy().to_string(),
        manifest.clone(),
    )
    .map_err(|_| AppBuildError::InternalError)?;

    let common = SageAppCommon::new(
        SageAppIdentity::new(
            spec.app_id.to_string(),
            spec.app_id.to_string(),
            app_dir.to_string_lossy().to_string(),
        )
        .map_err(|_| AppBuildError::InternalError)?,
        granted_permissions,
        builtin_storage(spec.app_id),
        snapshot,
        SageAppWalletScope::AllWallets,
    )
    .map_err(|_| AppBuildError::InternalError)?;

    let entry_file = app_dir.join(common.entry_file());
    if !entry_file.is_file() {
        return Err(AppBuildError::EntryFileNotFound);
    }

    let app = UserSageApp::new_installed(common, UserSageAppSource::Zip);

    Ok(Some(SageApp::User(app)))
}

pub fn build_builtin_runtime_app(app_id: &str) -> Result<Option<SageApp>, AppBuildError> {
    if app_id != BUILTIN_ORIGIN_CLEANUP_RUNTIME_ID {
        return Ok(None);
    }

    let app_dir = builtin_runtime_apps_root().join("origin-cleanup");

    if !app_dir.is_dir() {
        return Err(AppBuildError::AppDirMissing);
    }

    let manifest = read_builtin_manifest(&app_dir).map_err(|err| {
        tracing::error!(
            error = %err,
            app_dir = %app_dir.display(),
            "failed to build builtin runtime app manifest"
        );

        AppBuildError::ManifestFailure(format!("{err:#}"))
    })?;

    let granted_permissions = SageGrantedPermissions::for_builtin_requested(manifest.permissions())
        .map_err(|err| {
            tracing::error!("runtime app granted_permissions failed: {err}");
            AppBuildError::InternalError
        })?;

    let snapshot = SageAppSnapshot::new(
        format!("builtin-runtime:{app_id}"),
        app_dir.to_string_lossy().to_string(),
        manifest.clone(),
    )
    .map_err(|err| {
        tracing::error!("runtime app snapshot failed: {err}");
        AppBuildError::InternalError
    })?;

    let common = SageAppCommon::new(
        SageAppIdentity::new(app_id, app_id, app_dir.to_string_lossy().to_string()).map_err(
            |err| {
                tracing::error!("runtime app identity failed: {err}");
                AppBuildError::InternalError
            },
        )?,
        granted_permissions,
        SageAppStorage::Unmanaged,
        snapshot,
        SageAppWalletScope::AllWallets,
    )
    .map_err(|err| {
        tracing::error!("runtime app common failed: {err}");
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

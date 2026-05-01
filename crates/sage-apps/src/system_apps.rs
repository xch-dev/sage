use std::{
    fs,
    path::{Path, PathBuf},
};
use std::fmt::Display;
use anyhow::{Result as AnyResult};

use crate::bridge::capabilities::SystemBridgeCapability;
use crate::types::{
    InstalledSageAppStorage, SageApp, SageAppCommon, SageAppIdentity, SageAppPackageManifest,
    SageAppSnapshot, SageGrantedPermissions, SageGrantedSystemPermissions, SystemAppPresentation,
    SystemSageApp,
};
use crate::utils::builtin_apps_root;

pub const SYSTEM_APP_TASK_MANAGER_ID: &str = "task-manager";

#[derive(Debug, Clone, Copy)]
pub struct BuiltinSystemAppSpec {
    pub app_id: &'static str,
    pub dir_name: &'static str,
    pub presentation: SystemAppPresentation,
    pub system_capabilities: &'static [SystemBridgeCapability],
}

const BUILTIN_SYSTEM_APPS: &[BuiltinSystemAppSpec] = &[BuiltinSystemAppSpec {
    app_id: SYSTEM_APP_TASK_MANAGER_ID,
    dir_name: "task-manager",
    presentation: SystemAppPresentation::Taskbar,
    system_capabilities: &[
        SystemBridgeCapability::RuntimeManagerListRuntimes,
        SystemBridgeCapability::RuntimeManagerFocusRuntime,
        SystemBridgeCapability::RuntimeManagerHideRuntime,
        SystemBridgeCapability::RuntimeManagerKillRuntime,
        SystemBridgeCapability::RuntimeManagerListenRuntimesChanged,
    ],
}];

#[derive(Debug, Copy, Clone)]
pub enum AppBuildError {
    AppDirMissing,
    ManifestFailure,
    InternalError,
    EntryFileNotFound,
}

impl Display for AppBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            AppBuildError::AppDirMissing => String::from("app directory missing"),
            AppBuildError::ManifestFailure => String::from("manifest failure"),
            AppBuildError::InternalError => String::from("internal error"),
            AppBuildError::EntryFileNotFound => String::from("entry not found"),
        };
        write!(f, "{}", str)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum ReadBuiltinManifestError {
    NotFound,
    ParseFailed,
}

#[tauri::command]
#[specta::specta]
pub fn get_builtin_system_app(app_id: String) -> crate::host::Result<Option<crate::types::SageAppView>> {
    build_builtin_system_app(&app_id)
        .map(|app| app.map(|app| crate::types::SharedSageApp::new(app).into()))
        .map_err(|err| {
            std::io::Error::other(format!("failed to load builtin system app: {err}")).into()
        })
}

pub fn builtin_system_app_spec(app_id: &str) -> Option<&'static BuiltinSystemAppSpec> {
    BUILTIN_SYSTEM_APPS
        .iter()
        .find(|spec| spec.app_id == app_id)
}

pub fn builtin_system_apps_root() -> PathBuf {
    builtin_apps_root().join("system-apps")
}

pub fn builtin_system_app_dir(app_id: &str) -> AnyResult<Option<PathBuf>> {
    let Some(spec) = builtin_system_app_spec(app_id) else {
        return Ok(None);
    };

    Ok(Some(builtin_system_apps_root().join(spec.dir_name)))
}

fn read_builtin_manifest(app_dir: &Path) -> Result<SageAppPackageManifest, ReadBuiltinManifestError> {
    let manifest_path = app_dir.join("sage-manifest.json");

    let manifest_text = fs::read_to_string(&manifest_path).map_err(|_| ReadBuiltinManifestError::NotFound)?;

    serde_json::from_str(&manifest_text).map_err(|_| ReadBuiltinManifestError::ParseFailed)
}

pub fn build_builtin_system_app(app_id: &str) -> Result<Option<SageApp>, AppBuildError> {
    let Some(spec) = builtin_system_app_spec(app_id) else {
        return Ok(None);
    };

    let app_dir = builtin_system_apps_root().join(spec.dir_name);

    if !app_dir.is_dir() {
        eprintln!("[build_builtin_system_app] missing app_dir for {app_id}: {}", app_dir.display());
        return Err(AppBuildError::AppDirMissing);
    }

    let manifest = match read_builtin_manifest(&app_dir) {
        Ok(m) => m,
        Err(err) => {
            eprintln!(
                "[build_builtin_system_app] manifest failure for {app_id} at {}: {err:?}",
                app_dir.display()
            );
            return Err(AppBuildError::ManifestFailure);
        }
    };

    let requested_capabilities = manifest
        .permissions()
        .capabilities()
        .required()
        .chain(manifest.permissions().capabilities().optional())
        .copied()
        .filter(|capability| {
            crate::capabilities::get_user_capability_definition(*capability)
                .flags()
                .user_grantable()
        });

    let granted_permissions = match SageGrantedPermissions::new(
        manifest.permissions(),
        requested_capabilities,
        manifest
            .permissions()
            .network()
            .whitelist()
            .required()
            .cloned(),
    ) {
        Ok(p) => p,
        Err(err) => {
            eprintln!(
                "[build_builtin_system_app] granted_permissions failure for {app_id}: {err:?}\nmanifest: {manifest:?}"
            );
            return Err(AppBuildError::InternalError);
        }
    };

    let snapshot = match SageAppSnapshot::new_builtin_system(
        spec.app_id,
        app_dir.to_string_lossy().to_string(),
        manifest,
    ) {
        Ok(s) => s,
        Err(err) => {
            eprintln!(
                "[build_builtin_system_app] snapshot failure for {app_id}: {err:?}"
            );
            return Err(AppBuildError::InternalError);
        }
    };

    let identity = match SageAppIdentity::new(
        spec.app_id,
        spec.app_id,
        app_dir.to_string_lossy().to_string(),
    ) {
        Ok(i) => i,
        Err(err) => {
            eprintln!(
                "[build_builtin_system_app] identity failure for {app_id}: {err:?}"
            );
            return Err(AppBuildError::InternalError);
        }
    };

    let common = match SageAppCommon::new(
        identity,
        granted_permissions,
        InstalledSageAppStorage::Unmanaged,
        snapshot,
    ) {
        Ok(c) => c,
        Err(err) => {
            eprintln!(
                "[build_builtin_system_app] common failure for {app_id}: {err:?}"
            );
            return Err(AppBuildError::InternalError);
        }
    };

    let app = SystemSageApp::new(
        common,
        SageGrantedSystemPermissions::new(spec.system_capabilities.iter().copied()),
        spec.presentation,
    );

    Ok(Some(SageApp::System(app)))
}

pub fn list_builtin_system_apps() -> AnyResult<Vec<SageApp>> {
    let mut out = Vec::new();

    for spec in BUILTIN_SYSTEM_APPS {
        if let Some(app) = build_builtin_system_app(spec.app_id).map_err(|err| anyhow::anyhow!(format!("{err}")))? {
            out.push(app);
        }
    }

    Ok(out)
}

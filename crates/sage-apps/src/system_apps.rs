use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result as AnyResult, anyhow};
use tauri::command;

use crate::bridge::capabilities::SystemBridgeCapability;
use crate::host::Result;
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

fn read_builtin_manifest(app_dir: &Path) -> AnyResult<SageAppPackageManifest> {
    let manifest_path = app_dir.join("sage-manifest.json");

    let manifest_text = fs::read_to_string(&manifest_path).with_context(|| {
        format!(
            "failed to read builtin system app manifest {}",
            manifest_path.display()
        )
    })?;

    serde_json::from_str(&manifest_text).with_context(|| {
        format!(
            "failed to parse builtin system app manifest {}",
            manifest_path.display()
        )
    })
}

pub fn build_builtin_system_app(app_id: &str) -> AnyResult<Option<SageApp>> {
    let Some(spec) = builtin_system_app_spec(app_id) else {
        return Ok(None);
    };

    let app_dir = builtin_system_apps_root().join(spec.dir_name);

    if !app_dir.is_dir() {
        return Err(anyhow!(
            "builtin system app directory does not exist: {}",
            app_dir.display()
        ));
    }

    let manifest = read_builtin_manifest(&app_dir)?;

    let requested_capabilities = manifest
        .permissions()
        .capabilities()
        .required()
        .chain(manifest.permissions().capabilities().optional())
        .copied();

    let granted_permissions = SageGrantedPermissions::new(
        manifest.permissions(),
        requested_capabilities,
        manifest
            .permissions()
            .network()
            .whitelist()
            .required()
            .cloned(),
    )?;

    let snapshot = SageAppSnapshot::new_builtin_system(
        spec.app_id,
        app_dir.to_string_lossy().to_string(),
        manifest,
    )?;

    let common = SageAppCommon::new(
        SageAppIdentity::new(
            spec.app_id,
            spec.app_id,
            app_dir.to_string_lossy().to_string(),
        )?,
        granted_permissions,
        InstalledSageAppStorage::Unmanaged,
        snapshot,
    )?;

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
        if let Some(app) = build_builtin_system_app(spec.app_id)? {
            out.push(app);
        }
    }

    Ok(out)
}

#[command]
#[specta::specta]
pub fn get_builtin_system_app(app_id: &str) -> Result<Option<SageApp>> {
    build_builtin_system_app(app_id).map_err(|err| {
        std::io::Error::other(format!("failed to load builtin system app: {err}")).into()
    })
}

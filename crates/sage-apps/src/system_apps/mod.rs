use crate::bridge::capabilities::SystemBridgeCapability;
use crate::host::Result;
use crate::types::{
    InstalledSageAppStorage, SageApp, SageAppCommon, SageAppPackageManifest, SageAppSnapshot,
    SageGrantedPermissions, SageGrantedSystemPermissions, SystemAppPresentation, SystemSageApp,
};
use anyhow::{Context, Result as AnyResult, anyhow};
use std::path::Path;
use std::{fs, path::PathBuf};
use tauri::command;

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

pub fn builtin_apps_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("builtin-apps")
        .join("dist")
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

    let manifest: SageAppPackageManifest =
        serde_json::from_str(&manifest_text).with_context(|| {
            format!(
                "failed to parse builtin system app manifest {}",
                manifest_path.display()
            )
        })?;

    Ok(manifest)
}

fn compute_total_bytes(app_dir: &PathBuf) -> AnyResult<u64> {
    let mut total_bytes = 0_u64;

    for entry in fs::read_dir(app_dir).with_context(|| {
        format!(
            "failed to read builtin system app dir {}",
            app_dir.display()
        )
    })? {
        let entry = entry.with_context(|| {
            format!(
                "failed to read entry in builtin system app dir {}",
                app_dir.display()
            )
        })?;

        let metadata = entry.metadata().with_context(|| {
            format!(
                "failed to read metadata for builtin system app file {}",
                entry.path().display()
            )
        })?;

        if metadata.is_file() {
            total_bytes = total_bytes
                .checked_add(metadata.len())
                .ok_or_else(|| anyhow!("builtin system app total size overflow"))?;
        }
    }

    Ok(total_bytes)
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
        .capabilities
        .required()
        .chain(manifest.permissions().capabilities.optional())
        .copied();

    let granted_permissions = SageGrantedPermissions::new(
        manifest.permissions(),
        requested_capabilities,
        manifest.permissions().network.whitelist.required().cloned(),
    )?;

    let total_bytes = compute_total_bytes(&app_dir)?;

    let snapshot = SageAppSnapshot {
        manifest_hash: format!("builtin-system:{}", spec.app_id),
        snapshot_dir: app_dir.to_string_lossy().to_string(),
        total_bytes,
        manifest: manifest.clone(),
    };

    let common = SageAppCommon::new(
        spec.app_id.to_string(),
        spec.app_id.to_string(),
        app_dir.to_string_lossy().to_string(),
        &manifest,
        granted_permissions,
        InstalledSageAppStorage::Unmanaged,
        snapshot,
    )?;

    let entry_file = app_dir.join(&common.entry_file);
    if !entry_file.is_file() {
        return Err(anyhow!(
            "builtin system app entry file does not exist: {}",
            entry_file.display()
        ));
    }

    let icon_file = app_dir.join(&common.icon_file);
    if !icon_file.is_file() {
        return Err(anyhow!(
            "builtin system app icon file does not exist: {}",
            icon_file.display()
        ));
    }

    let app = SystemSageApp {
        common,
        presentation: spec.presentation,
        system_granted_permissions: SageGrantedSystemPermissions {
            capabilities: spec.system_capabilities.to_vec(),
        },
    };

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

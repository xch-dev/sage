use crate::capabilities::list::{SystemBridgeCapability, UserBridgeCapability};
use crate::types::{
    SageAppStorage, SageApp, SageAppCommon, SageAppIdentity, SageAppPackageManifest,
    SageAppSnapshot, SageAppWalletScope, SageGrantedPermissions, SageGrantedSystemPermissions,
    SystemSageApp,
};
use crate::utils::builtin_apps_root;
use anyhow::Result as AnyResult;
use serde::Serialize;
use specta::Type;
use std::collections::BTreeMap;
use std::fmt::Display;
use std::{
    fs,
    path::{Path, PathBuf},
};

pub const SYSTEM_APP_TASK_MANAGER_ID: &str = "task-manager";
pub const SYSTEM_APP_APP_UPDATE_ID: &str = "app-update";
pub const SYSTEM_APP_APP_INSTALL_ID: &str = "app-install";
pub const SYSTEM_APP_BRIDGE_APPROVAL_ID: &str = "bridge-approval";
pub const SYSTEM_APP_DONATION_ID: &str = "donation";
pub const SYSTEM_APP_SANDBOX_TESTS_ID: &str = "sandbox-tests";

#[derive(Debug, Clone, Copy)]
pub struct BuiltinSystemAppSpec {
    pub app_id: &'static str,
    pub dir_name: &'static str,
    pub usage: SystemAppUsage,
    pub system_capabilities: &'static [SystemBridgeCapability],
    pub user_grantable_capabilities: &'static [UserBridgeCapability],
}
#[derive(Debug, Clone, Copy, Serialize, Type, PartialEq, Eq)]
pub enum SystemAppUsage {
    Standalone,
    Contextual,
}

const BUILTIN_SYSTEM_APPS: &[BuiltinSystemAppSpec] = &[
    BuiltinSystemAppSpec {
        app_id: SYSTEM_APP_TASK_MANAGER_ID,
        dir_name: "task-manager",
        usage: SystemAppUsage::Standalone,
        system_capabilities: &[
            SystemBridgeCapability::RuntimeManagerListRuntimes,
            SystemBridgeCapability::RuntimeManagerFocusTaskbarRuntime,
            SystemBridgeCapability::RuntimeManagerHideRuntime,
            SystemBridgeCapability::RuntimeManagerKillRuntime,
            SystemBridgeCapability::RuntimeManagerListenRuntimesChanged,
        ],
        user_grantable_capabilities: &[],
    },
    BuiltinSystemAppSpec {
        app_id: SYSTEM_APP_APP_UPDATE_ID,
        dir_name: "app-update",
        usage: SystemAppUsage::Contextual,
        system_capabilities: &[
            SystemBridgeCapability::CapabilityDefinitionsRead,
            SystemBridgeCapability::AppPermissionsRead,
            SystemBridgeCapability::AppPermissionsApply,
            SystemBridgeCapability::AppUpdateRead,
            SystemBridgeCapability::AppUpdateApply,
            SystemBridgeCapability::WalletListWallets,
            SystemBridgeCapability::RuntimeManagerCloseSelf,
        ],
        user_grantable_capabilities: &[],
    },
    BuiltinSystemAppSpec {
        app_id: SYSTEM_APP_APP_INSTALL_ID,
        dir_name: "app-install",
        usage: SystemAppUsage::Contextual,
        system_capabilities: &[
            SystemBridgeCapability::CapabilityDefinitionsRead,
            SystemBridgeCapability::AppInstallPreview,
            SystemBridgeCapability::AppInstallApply,
            SystemBridgeCapability::FileSystemSelectFile,
            SystemBridgeCapability::WalletListWallets,
            SystemBridgeCapability::RuntimeManagerCloseSelf,
        ],
        user_grantable_capabilities: &[],
    },
    BuiltinSystemAppSpec {
        app_id: SYSTEM_APP_BRIDGE_APPROVAL_ID,
        dir_name: "bridge-approval",
        usage: SystemAppUsage::Contextual,
        system_capabilities: &[
            SystemBridgeCapability::BridgeApprovalList,
            SystemBridgeCapability::BridgeApprovalResolve,
            SystemBridgeCapability::BridgeApprovalListenApprovalsChanged,
            SystemBridgeCapability::RuntimeManagerGetActiveTaskbarRuntime,
            SystemBridgeCapability::RuntimeManagerHideSelf,
            SystemBridgeCapability::RuntimeManagerCloseSelf,
        ],
        user_grantable_capabilities: &[],
    },
    BuiltinSystemAppSpec {
        app_id: SYSTEM_APP_DONATION_ID,
        dir_name: "donation",
        usage: SystemAppUsage::Contextual,
        system_capabilities: &[
            SystemBridgeCapability::DonationGetDetails,
            SystemBridgeCapability::RuntimeManagerCloseSelf,
        ],
        user_grantable_capabilities: &[UserBridgeCapability::WalletSendXchAutoSubmit],
    },
    BuiltinSystemAppSpec {
        app_id: SYSTEM_APP_SANDBOX_TESTS_ID,
        dir_name: "sandbox-tests",
        usage: SystemAppUsage::Contextual,
        system_capabilities: &[
            SystemBridgeCapability::SandboxGetState,
            SystemBridgeCapability::SandboxRerunTests,
            SystemBridgeCapability::SandboxListenStateChanged,
            SystemBridgeCapability::RuntimeManagerCloseSelf,
        ],
        user_grantable_capabilities: &[],
    },
];

#[derive(Debug, Clone)]
pub enum AppBuildError {
    AppDirMissing,
    ManifestFailure(String),
    InternalError,
    EntryFileNotFound,
}

impl Display for AppBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppBuildError::AppDirMissing => write!(f, "app directory is missing"),
            AppBuildError::ManifestFailure(err) => write!(f, "manifest failure: {err}"),
            AppBuildError::EntryFileNotFound => write!(f, "entry file not found"),
            AppBuildError::InternalError => write!(f, "internal app build error"),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum ReadBuiltinManifestError {
    NotFound,
    ParseFailed,
}

pub fn builtin_system_app_spec(app_id: &str) -> Option<&'static BuiltinSystemAppSpec> {
    BUILTIN_SYSTEM_APPS
        .iter()
        .find(|spec| spec.app_id == app_id)
}

pub fn builtin_system_apps_root() -> PathBuf {
    builtin_apps_root().join("system")
}

fn read_builtin_manifest(
    app_dir: &Path,
) -> Result<SageAppPackageManifest, ReadBuiltinManifestError> {
    let manifest_path = app_dir.join("sage-manifest.json");

    let manifest_text =
        fs::read_to_string(&manifest_path).map_err(|_| ReadBuiltinManifestError::NotFound)?;

    serde_json::from_str(&manifest_text).map_err(|_| ReadBuiltinManifestError::ParseFailed)
}

pub fn build_builtin_system_app(app_id: &str) -> Result<Option<SageApp>, AppBuildError> {
    let Some(spec) = builtin_system_app_spec(app_id) else {
        return Ok(None);
    };

    let app_dir = builtin_system_apps_root().join(spec.dir_name);

    if !app_dir.is_dir() {
        tracing::error!(
            "[build_builtin_system_app] missing app_dir for {app_id}: {}",
            app_dir.display()
        );
        return Err(AppBuildError::AppDirMissing);
    }

    let manifest = match read_builtin_manifest(&app_dir) {
        Ok(m) => m,
        Err(err) => {
            tracing::error!(
                "[build_builtin_system_app] manifest failure for {app_id} at {}: {err:?}",
                app_dir.display()
            );
            return Err(AppBuildError::ManifestFailure(format!("{err:?}")));
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

    let granted_permissions = match SageGrantedPermissions::new_with_extra_granted_capabilities(
        manifest.permissions(),
        requested_capabilities,
        spec.user_grantable_capabilities.iter().copied(),
        manifest
            .permissions()
            .network()
            .whitelist()
            .required()
            .cloned(),
        BTreeMap::new(),
    ) {
        Ok(p) => p,
        Err(err) => {
            tracing::error!(
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
            tracing::error!("[build_builtin_system_app] snapshot failure for {app_id}: {err:?}");
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
            tracing::error!("[build_builtin_system_app] identity failure for {app_id}: {err:?}");
            return Err(AppBuildError::InternalError);
        }
    };

    let common = match SageAppCommon::new(
        identity,
        granted_permissions,
        SageAppStorage::Unmanaged,
        snapshot,
        SageAppWalletScope::AllWallets,
    ) {
        Ok(c) => c,
        Err(err) => {
            tracing::error!("[build_builtin_system_app] common failure for {app_id}: {err:?}");
            return Err(AppBuildError::InternalError);
        }
    };

    let app = SystemSageApp::new(
        common,
        spec.usage,
        SageGrantedSystemPermissions::new(spec.system_capabilities.iter().copied()),
    );

    Ok(Some(SageApp::System(app)))
}

pub fn list_builtin_system_apps() -> AnyResult<Vec<SageApp>> {
    let mut out = Vec::new();

    for spec in BUILTIN_SYSTEM_APPS {
        if let Some(app) = build_builtin_system_app(spec.app_id)
            .map_err(|err| anyhow::anyhow!(format!("{err}")))?
        {
            out.push(app);
        }
    }

    Ok(out)
}

use semver::Version;

use super::{AppLaunchGateResult, SandboxCapability, SandboxCapabilityStatus, SandboxState};
use crate::{
    SageAppCompatibility, SageAppCompatibilityStatus, SharedSageApp, UserBridgeCapability,
};

fn capability_status(
    state: &SandboxState,
    capability: SandboxCapability,
) -> SandboxCapabilityStatus {
    match capability {
        SandboxCapability::StorageIsolationFromSage => state.storage_isolation_from_sage.status,
        SandboxCapability::StoragePersistenceNormal => state.storage_persistence_normal.status,
        SandboxCapability::StorageNonPersistenceIncognito => {
            state.storage_non_persistence_incognito.status
        }
        SandboxCapability::NetworkAllowlistEnforced => state.network_allowlist_enforced.status,
    }
}

fn required_capabilities_for_app(app: &SharedSageApp) -> Vec<SandboxCapability> {
    let mut caps = vec![
        SandboxCapability::StorageIsolationFromSage,
        SandboxCapability::NetworkAllowlistEnforced,
    ];

    if app.is_capability_granted(UserBridgeCapability::StoragePersistentWebview.into()) {
        caps.push(SandboxCapability::StoragePersistenceNormal);
    } else {
        caps.push(SandboxCapability::StorageNonPersistenceIncognito);
    }

    caps
}

pub fn evaluate_app_launch_gate(
    app: &SharedSageApp,
    effective: &SandboxState,
    current_sage_version: &Version,
) -> AppLaunchGateResult {
    if app.is_system_app() || app.id().starts_with("__sage_test_") {
        return AppLaunchGateResult {
            allowed: true,
            kind: "allowed".into(),
            capability: None,
            message: None,
        };
    }

    let compatibility = app.with(|app| {
        SageAppCompatibility::evaluate(
            current_sage_version,
            app.common().active_manifest().sage_version(),
        )
    });

    let compatibility_warning = match compatibility.status() {
        SageAppCompatibilityStatus::RequiresNewerSage { minimum_version } => {
            return AppLaunchGateResult {
                allowed: false,
                kind: "requiresNewerSage".into(),
                capability: None,
                message: Some(format!(
                    "This app requires Sage {minimum_version} or newer. You are running Sage {current_sage_version}."
                )),
            };
        }
        SageAppCompatibilityStatus::Invalid { reason } => {
            return AppLaunchGateResult {
                allowed: false,
                kind: "invalidSageVersion".into(),
                capability: None,
                message: Some(format!(
                    "This app has an invalid Sage version requirement: {reason}"
                )),
            };
        }
        SageAppCompatibilityStatus::UntestedNewerSage { tested_max_version } => Some(format!(
            "This app has only been tested through Sage {tested_max_version}. You are running Sage {current_sage_version}."
        )),
        SageAppCompatibilityStatus::Compatible => None,
    };

    for capability in required_capabilities_for_app(app) {
        match capability_status(effective, capability) {
            SandboxCapabilityStatus::Passed => {}
            SandboxCapabilityStatus::Pending | SandboxCapabilityStatus::Running => {
                return AppLaunchGateResult {
                    allowed: false,
                    kind: "sandboxPending".into(),
                    capability: Some(capability),
                    message: Some(
                        "Apps are allowed to launch only when all required sandbox capabilities have passed."
                            .into(),
                    ),
                };
            }
            SandboxCapabilityStatus::Failed => {
                return AppLaunchGateResult {
                    allowed: false,
                    kind: "sandboxFailed".into(),
                    capability: Some(capability),
                    message: Some(
                        "Apps are blocked because a required sandbox capability failed.".into(),
                    ),
                };
            }
        }
    }

    match compatibility_warning {
        Some(message) => AppLaunchGateResult {
            allowed: true,
            kind: "sageUntested".into(),
            capability: None,
            message: Some(message),
        },
        None => AppLaunchGateResult {
            allowed: true,
            kind: "allowed".into(),
            capability: None,
            message: None,
        },
    }
}

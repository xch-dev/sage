use crate::bridge::capabilities::UserBridgeCapability;
use crate::types::{SharedSageApp};

use super::{AppLaunchGateResult, SandboxCapability, SandboxCapabilityStatus, SandboxState};

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
        SandboxCapability::StorageClearCycle => state.storage_clear_cycle.status,
        SandboxCapability::NetworkAllowlistEnforced => state.network_allowlist_enforced.status,
    }
}

fn required_capabilities_for_app(app: &SharedSageApp) -> Vec<SandboxCapability> {
    let mut caps = vec![
        SandboxCapability::StorageIsolationFromSage,
        SandboxCapability::NetworkAllowlistEnforced,
    ];

    if app.is_capability_granted(UserBridgeCapability::PersistentStorage) {
        caps.push(SandboxCapability::StoragePersistenceNormal);

        if app.has_secret_access() {
            caps.push(SandboxCapability::StorageClearCycle);
        }
    } else {
        caps.push(SandboxCapability::StorageNonPersistenceIncognito);
    }

    caps
}

pub fn evaluate_app_launch_gate(app: &SharedSageApp, effective: &SandboxState) -> AppLaunchGateResult {
    if app.is_system_app() || app.id().starts_with("__sage_test_") {
        return AppLaunchGateResult {
            allowed: true,
            kind: "allowed".into(),
            capability: None,
            message: None,
        };
    }

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

    AppLaunchGateResult {
        allowed: true,
        kind: "allowed".into(),
        capability: None,
        message: None,
    }
}

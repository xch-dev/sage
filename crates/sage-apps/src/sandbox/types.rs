use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum SandboxCapability {
    StorageIsolationFromSage,
    StoragePersistenceNormal,
    StorageNonPersistenceIncognito,
    StorageClearCycle,
    NetworkAllowlistEnforced,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SandboxCapabilityStatus {
    Pending,
    Running,
    Passed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SandboxCapabilityResult {
    pub status: SandboxCapabilityStatus,
    pub checked_at: Option<i64>,
    pub details: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SandboxState {
    pub overall_critical_status: SandboxCapabilityStatus,
    pub storage_isolation_from_sage: SandboxCapabilityResult,
    pub storage_persistence_normal: SandboxCapabilityResult,
    pub storage_non_persistence_incognito: SandboxCapabilityResult,
    pub storage_clear_cycle: SandboxCapabilityResult,
    pub network_allowlist_enforced: SandboxCapabilityResult,
    pub started_at: Option<i64>,
    pub finished_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SandboxRunState {
    pub run_id: String,
    pub state: SandboxState,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SandboxStateView {
    pub baseline: SandboxState,
    pub current_run: Option<SandboxRunState>,
    pub effective: SandboxState,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AppLaunchGateResult {
    pub allowed: bool,
    pub kind: String,
    pub capability: Option<SandboxCapability>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SandboxIsolationProbeResult {
    pub run_id: String,
    pub local_storage_visible: bool,
    pub indexed_db_visible: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SandboxPersistenceWriteProbeResult {
    pub run_id: String,
    pub local_storage_wrote: bool,
    pub indexed_db_wrote: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SandboxPersistenceReadProbeResult {
    pub run_id: String,
    pub local_storage_present: bool,
    pub indexed_db_present: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SandboxNetworkProbeResult {
    pub run_id: String,
    pub allowed_url: String,
    pub blocked_url: String,
    pub allowed_ok: bool,
    pub blocked_ok: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SandboxStorageClearProbeResult {
    pub run_id: String,
    pub phase: SandboxStorageClearProbePhase,
    pub local_storage_present: bool,
    pub indexed_db_present: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SandboxStorageClearProbePhase {
    Write,
    CheckPresent,
    CheckAbsent,
}

pub fn make_cap(
    status: SandboxCapabilityStatus,
    details: Option<String>,
) -> SandboxCapabilityResult {
    SandboxCapabilityResult {
        status,
        checked_at: None,
        details,
    }
}

pub fn mark_cap(
    state: &mut SandboxState,
    cap: SandboxCapability,
    status: SandboxCapabilityStatus,
    details: Option<String>,
    checked_at: i64,
) {
    let next = SandboxCapabilityResult {
        status,
        checked_at: Some(checked_at),
        details,
    };

    match cap {
        SandboxCapability::StorageIsolationFromSage => {
            state.storage_isolation_from_sage = next;
        }
        SandboxCapability::StoragePersistenceNormal => {
            state.storage_persistence_normal = next;
        }
        SandboxCapability::StorageNonPersistenceIncognito => {
            state.storage_non_persistence_incognito = next;
        }
        SandboxCapability::StorageClearCycle => state.storage_clear_cycle = next,
        SandboxCapability::NetworkAllowlistEnforced => {
            state.network_allowlist_enforced = next;
        }
    }
}

pub fn build_initial_sandbox_state() -> SandboxState {
    SandboxState {
        overall_critical_status: SandboxCapabilityStatus::Pending,
        storage_isolation_from_sage: make_cap(SandboxCapabilityStatus::Pending, None),
        storage_persistence_normal: make_cap(SandboxCapabilityStatus::Pending, None),
        storage_non_persistence_incognito: make_cap(SandboxCapabilityStatus::Pending, None),
        storage_clear_cycle: make_cap(SandboxCapabilityStatus::Pending, None),
        network_allowlist_enforced: make_cap(SandboxCapabilityStatus::Pending, None),
        started_at: None,
        finished_at: None,
    }
}

pub fn build_running_sandbox_state(started_at: i64) -> SandboxState {
    SandboxState {
        overall_critical_status: SandboxCapabilityStatus::Running,
        storage_isolation_from_sage: make_cap(SandboxCapabilityStatus::Running, None),
        storage_persistence_normal: make_cap(SandboxCapabilityStatus::Running, None),
        storage_non_persistence_incognito: make_cap(SandboxCapabilityStatus::Running, None),
        storage_clear_cycle: make_cap(SandboxCapabilityStatus::Running, None),
        network_allowlist_enforced: make_cap(SandboxCapabilityStatus::Running, None),
        started_at: Some(started_at),
        finished_at: None,
    }
}

use std::collections::HashMap;

use tokio::sync::Mutex;

use super::types::{
    SandboxIsolationProbeResult, SandboxNetworkProbeResult, SandboxPersistenceReadProbeResult,
    SandboxPersistenceWriteProbeResult, SandboxRunState, SandboxStorageClearProbeResult,
    build_initial_sandbox_state,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SandboxAppResult<T> {
    pub app_id: String,
    pub data: T,
}

#[derive(Debug, Clone, Default)]
pub struct SandboxRunResults {
    pub isolation: Vec<SandboxAppResult<SandboxIsolationProbeResult>>,
    pub persistence_write: Vec<SandboxAppResult<SandboxPersistenceWriteProbeResult>>,
    pub persistence_read: Vec<SandboxAppResult<SandboxPersistenceReadProbeResult>>,
    pub clear_cycle: Vec<SandboxAppResult<SandboxStorageClearProbeResult>>,
    pub network: Vec<SandboxAppResult<SandboxNetworkProbeResult>>,
}

#[derive(Debug)]
pub struct SandboxStateStore {
    pub baseline: Mutex<super::types::SandboxState>,
    pub current_run: Mutex<Option<SandboxRunState>>,
    pub runs: Mutex<HashMap<String, SandboxRunResults>>,
    pub running: Mutex<bool>,
}

impl Default for SandboxStateStore {
    fn default() -> Self {
        Self {
            baseline: Mutex::new(build_initial_sandbox_state()),
            current_run: Mutex::new(None),
            runs: Mutex::new(HashMap::new()),
            running: Mutex::new(false),
        }
    }
}

pub fn replace_by_app_id<T>(items: &mut Vec<SandboxAppResult<T>>, next: SandboxAppResult<T>) {
    if let Some(existing) = items.iter_mut().find(|item| item.app_id == next.app_id) {
        *existing = next;
    } else {
        items.push(next);
    }
}

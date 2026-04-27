use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::{BTreeMap, BTreeSet};
use tokio::sync::{Mutex, oneshot};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SageAppRuntimeKind {
    User,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SageLifecycleBeforeStopDetail {
    pub request_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime_id: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SetBeforeStopListenerParams {
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ReadyToStopParams {
    pub request_id: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeAckResult {
    pub ok: bool,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SageAppRuntimeRecord {
    pub runtime_id: String,
    pub app_id: String,
    pub app_name: String,
    pub entry_src: String,
    pub webview_label: String,
    pub host_window_label: String,
    pub runtime_kind: SageAppRuntimeKind,
    pub mode: String,
    pub state: String,
    pub started_at: i64,
    pub last_active_at: i64,
    pub visible: bool,
    pub internal: bool,
}

#[derive(Default)]
pub struct AppRuntimeState {
    pub runtime_by_runtime_id: Mutex<BTreeMap<String, SageAppRuntimeRecord>>,
    pub runtime_id_by_app_id: Mutex<BTreeMap<String, String>>,
    pub before_stop_listeners_by_app_id: Mutex<BTreeSet<String>>,
    pub pending_stop_ready: Mutex<BTreeMap<String, oneshot::Sender<()>>>,
}

impl std::fmt::Debug for AppRuntimeState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppRuntimeState").finish()
    }
}

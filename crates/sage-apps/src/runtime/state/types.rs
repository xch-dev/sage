use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use specta::Type;
use tokio::sync::{Mutex, oneshot};

use crate::bridge::capabilities::UserBridgeCapability;
use crate::runtime::runtime_kind_for_app;
use crate::types::SageApp;
use crate::utils::unix_timestamp_ms;

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
    active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ReadyToStopParams {
    request_id: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeAckResult {
    pub ok: bool,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SageAppRuntimeRecord {
    runtime_id: String,
    app_id: String,
    app_name: String,
    entry_src: String,
    webview_label: String,
    host_window_label: String,
    runtime_kind: SageAppRuntimeKind,
    mode: String,
    state: String,
    started_at: i64,
    last_active_at: i64,
    visible: bool,
    internal: bool,
}

#[derive(Default)]
pub struct AppRuntimeState {
    pub runtime_by_runtime_id: Mutex<BTreeMap<String, SageAppRuntimeRecord>>,
    pub runtime_id_by_app_id: Mutex<BTreeMap<String, String>>,
    pub before_stop_listeners_by_app_id: Mutex<BTreeSet<String>>,
    pub pending_stop_ready: Mutex<BTreeMap<String, oneshot::Sender<()>>>,
}

impl SageLifecycleBeforeStopDetail {
    pub fn new(
        request_id: impl Into<String>,
        reason: Option<impl Into<String>>,
        app_id: Option<impl Into<String>>,
        runtime_id: Option<impl Into<String>>,
    ) -> Self {
        Self {
            request_id: request_id.into(),
            reason: reason.map(Into::into),
            app_id: app_id.map(Into::into),
            runtime_id: runtime_id.map(Into::into),
        }
    }

    pub fn request_id(&self) -> &str {
        &self.request_id
    }

    pub fn reason(&self) -> Option<&str> {
        self.reason.as_deref()
    }

    pub fn app_id(&self) -> Option<&str> {
        self.app_id.as_deref()
    }

    pub fn runtime_id(&self) -> Option<&str> {
        self.runtime_id.as_deref()
    }
}

impl SetBeforeStopListenerParams {
    pub fn active(&self) -> bool {
        self.active
    }
}

impl ReadyToStopParams {
    pub fn request_id(&self) -> &str {
        &self.request_id
    }
}

impl RuntimeAckResult {
    pub fn ok() -> Self {
        Self { ok: true }
    }

    pub fn ok_value(&self) -> bool {
        self.ok
    }
}

impl SageAppRuntimeRecord {
    pub fn new_inline(
        app: &mut SageApp,
        entry_src: impl Into<String>,
        visible: bool,
        internal: bool,
    ) -> Self {
        app.taint_storage_if_runtime_can_persist_secrets();

        let runtime_kind = runtime_kind_for_app(app);
        let now = unix_timestamp_ms();

        Self {
            runtime_id: runtime_id_for(app.id(), runtime_kind),
            app_id: app.id().to_string(),
            app_name: app.name().to_string(),
            entry_src: entry_src.into(),
            webview_label: inline_label_for(app.id(), runtime_kind),
            host_window_label: "main".into(),
            runtime_kind,
            mode: "inline".into(),
            state: runtime_state_for_visible(visible).into(),
            started_at: now,
            last_active_at: now,
            visible,
            internal,
        }
    }

    pub fn new_existing_inline_fallback(
        runtime_id: impl Into<String>,
        app_id: impl Into<String>,
        app_name: impl Into<String>,
        entry_src: impl Into<String>,
        webview_label: impl Into<String>,
        runtime_kind: SageAppRuntimeKind,
        internal: bool,
    ) -> Self {
        let now = unix_timestamp_ms();

        Self {
            runtime_id: runtime_id.into(),
            app_id: app_id.into(),
            app_name: app_name.into(),
            entry_src: entry_src.into(),
            webview_label: webview_label.into(),
            host_window_label: "main".into(),
            runtime_kind,
            mode: "inline".into(),
            state: "hidden".into(),
            started_at: now,
            last_active_at: now,
            visible: false,
            internal,
        }
    }

    pub fn mark_inline_reused(&mut self, visible: bool, internal: bool) {
        self.visible = visible;
        self.state = runtime_state_for_visible(visible).into();
        self.last_active_at = unix_timestamp_ms();
        self.internal = internal;
    }

    pub fn mark_visible(&mut self) {
        self.visible = true;
        self.state = "running".into();
        self.last_active_at = unix_timestamp_ms();
    }

    pub fn mark_hidden(&mut self) {
        self.visible = false;
        self.state = "hidden".into();
        self.last_active_at = unix_timestamp_ms();
    }

    pub fn runtime_id(&self) -> &str {
        &self.runtime_id
    }

    pub fn app_id(&self) -> &str {
        &self.app_id
    }

    pub fn app_name(&self) -> &str {
        &self.app_name
    }

    pub fn entry_src(&self) -> &str {
        &self.entry_src
    }

    pub fn webview_label(&self) -> &str {
        &self.webview_label
    }

    pub fn host_window_label(&self) -> &str {
        &self.host_window_label
    }

    pub fn runtime_kind(&self) -> SageAppRuntimeKind {
        self.runtime_kind
    }

    pub fn mode(&self) -> &str {
        &self.mode
    }

    pub fn state(&self) -> &str {
        &self.state
    }

    pub fn started_at(&self) -> i64 {
        self.started_at
    }

    pub fn last_active_at(&self) -> i64 {
        self.last_active_at
    }

    pub fn visible(&self) -> bool {
        self.visible
    }

    pub fn internal(&self) -> bool {
        self.internal
    }
}

impl SageApp {
    pub(crate) fn taint_storage_if_runtime_can_persist_secrets(&mut self) {
        let has_persistent_storage = self
            .granted_permissions()
            .capabilities()
            .any(|cap| *cap == UserBridgeCapability::PersistentStorage);

        if self.flags().has_secret_access()
            && has_persistent_storage
            && !self.flags().storage_may_contain_secrets()
        {
            self.common_mut().mark_storage_may_contain_secrets();
        }
    }
}

impl std::fmt::Debug for AppRuntimeState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppRuntimeState").finish()
    }
}

pub fn runtime_id_for(app_id: &str, runtime_kind: SageAppRuntimeKind) -> String {
    match runtime_kind {
        SageAppRuntimeKind::User => format!("runtime-{app_id}"),
        SageAppRuntimeKind::System => format!("system-runtime-{app_id}"),
    }
}

pub fn inline_label_for(app_id: &str, runtime_kind: SageAppRuntimeKind) -> String {
    match runtime_kind {
        SageAppRuntimeKind::User => format!("app-inline-{app_id}"),
        SageAppRuntimeKind::System => format!("system-app-inline-{app_id}"),
    }
}

fn runtime_state_for_visible(visible: bool) -> &'static str {
    if visible { "running" } else { "hidden" }
}

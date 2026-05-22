use crate::types::{AppPresentation, SageApp, SharedSageApp};
use crate::utils::unix_timestamp_ms;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use tokio::sync::{Mutex, oneshot};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq)]
pub enum SageAppRuntimeMode {
    Inline,
    Windowed,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq)]
pub enum SageAppRuntimeVisibility {
    Visible,
    Hidden,
}

#[derive(Debug)]
pub struct SageAppRuntimeRecord {
    runtime_id: String,
    app: SharedSageApp,
    host_window_label: String,
    webview_label: String,
    presentation: AppPresentation,
    mode: SageAppRuntimeMode,
    visibility: SageAppRuntimeVisibility,
    started_at: i64,
    last_active_at: i64,
    internal: bool,
}

#[derive(Debug, Clone)]
pub struct SharedRuntime {
    inner: Arc<RwLock<SageAppRuntimeRecord>>,
}

#[derive(Default)]
pub struct AppRuntimeState {
    pub apps_workspace_active: tokio::sync::RwLock<bool>,

    pub runtime_by_runtime_id: Mutex<BTreeMap<String, SharedRuntime>>,
    pub runtime_id_by_app_id: Mutex<BTreeMap<String, String>>,

    pub before_stop_listeners_by_app_id: Mutex<BTreeSet<String>>,
    pub pending_stop_ready: Mutex<BTreeMap<String, oneshot::Sender<()>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SageLifecycleBeforeStopDetail {
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
pub(crate) struct SetBeforeStopListenerParams {
    active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ReadyToStopParams {
    request_id: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RuntimeAckResult {
    pub ok: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreateRuntimeRecordError {
    UserAppCannotUseModalPresentation,
}

impl std::fmt::Display for CreateRuntimeRecordError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UserAppCannotUseModalPresentation => {
                write!(f, "user app cannot use modal presentation")
            }
        }
    }
}

pub(in crate::runtime) fn runtime_id_for(app: &SharedSageApp) -> String {
    let (app_id, is_system_app) = app.with(|app| (app.id().to_string(), app.is_system()));
    if is_system_app {
        return format!("system-runtime-{app_id}");
    }

    format!("runtime-{app_id}")
}

impl SetBeforeStopListenerParams {
    pub fn active(self) -> bool {
        self.active
    }
}

impl ReadyToStopParams {
    pub fn request_id(&self) -> &str {
        &self.request_id
    }
}

impl SageAppRuntimeRecord {
    pub(crate) fn new(
        app: &SharedSageApp,
        host_window_label: &str,
        webview_label: &str,
        presentation: AppPresentation,
        mode: SageAppRuntimeMode,
        visibility: SageAppRuntimeVisibility,
        internal: bool,
    ) -> Result<Self, CreateRuntimeRecordError> {
        if app.is_user_app() && matches!(presentation, AppPresentation::Modal(_)) {
            return Err(CreateRuntimeRecordError::UserAppCannotUseModalPresentation);
        }

        let now = unix_timestamp_ms();

        Ok(Self {
            runtime_id: runtime_id_for(app),
            app: app.clone(),
            host_window_label: host_window_label.to_string(),
            webview_label: webview_label.to_string(),
            presentation,
            mode,
            visibility,
            started_at: now,
            last_active_at: now,
            internal,
        })
    }

    pub(crate) fn mark_visible(&mut self) {
        self.visibility = SageAppRuntimeVisibility::Visible;
        self.last_active_at = unix_timestamp_ms();
    }

    pub(crate) fn mark_hidden(&mut self) {
        self.visibility = SageAppRuntimeVisibility::Hidden;
        self.last_active_at = unix_timestamp_ms();
    }

    pub(crate) fn runtime_id(&self) -> String {
        self.runtime_id.to_string()
    }

    pub(crate) fn app(&self) -> SharedSageApp {
        self.app.clone()
    }

    pub(crate) fn app_id(&self) -> String {
        self.app.id()
    }

    pub(crate) fn webview_label(&self) -> String {
        self.webview_label.to_string()
    }

    pub(crate) fn host_window_label(&self) -> String {
        self.host_window_label.to_string()
    }

    pub(crate) fn presentation(&self) -> AppPresentation {
        self.presentation.clone()
    }

    pub(crate) fn update_modal_presentation_list(
        &mut self,
        target_app_ids: Vec<String>,
    ) -> Result<bool, String> {
        match &mut self.presentation {
            AppPresentation::Modal(presentation) => Ok(presentation.update_app_ids(target_app_ids)),
            AppPresentation::Taskbar => Err("Presentation mode is not modal".to_string()),
        }
    }

    pub(crate) fn mode(&self) -> SageAppRuntimeMode {
        self.mode
    }

    pub(crate) fn visibility(&self) -> SageAppRuntimeVisibility {
        self.visibility
    }

    pub(crate) fn started_at(&self) -> i64 {
        self.started_at
    }

    pub(crate) fn last_active_at(&self) -> i64 {
        self.last_active_at
    }

    pub(crate) fn internal(&self) -> bool {
        self.internal
    }
}

impl SharedSageApp {
    pub fn webview_label(&self) -> String {
        self.with(|app| {
            if app.is_system() {
                format!("system-app-{}", app.id())
            } else {
                format!("app-{}", app.id())
            }
        })
    }
}

impl std::fmt::Debug for AppRuntimeState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppRuntimeState").finish()
    }
}

impl SharedRuntime {
    pub fn new(runtime: SageAppRuntimeRecord) -> Self {
        Self {
            inner: Arc::new(RwLock::new(runtime)),
        }
    }

    pub fn with_runtime<T>(&self, f: impl FnOnce(&SageAppRuntimeRecord) -> T) -> T {
        let runtime = self.inner.read();
        f(&runtime)
    }

    pub fn with_runtime_mut<T>(&self, f: impl FnOnce(&mut SageAppRuntimeRecord) -> T) -> T {
        let mut runtime = self.inner.write();
        f(&mut runtime)
    }

    pub fn app(&self) -> SharedSageApp {
        self.with_runtime(|runtime| runtime.app.clone())
    }

    pub fn with_app<T>(&self, f: impl FnOnce(&SharedSageApp) -> T) -> T {
        let app = self.app();
        f(&app)
    }

    pub fn with_app_inner<T>(&self, f: impl FnOnce(&SageApp) -> T) -> T {
        self.with_runtime(|runtime| runtime.app().with(f))
    }

    pub fn is_user_app(&self) -> bool {
        self.with_app(SharedSageApp::is_user_app)
    }

    pub fn is_system_app(&self) -> bool {
        self.with_app(SharedSageApp::is_system_app)
    }

    pub fn runtime_id(&self) -> String {
        self.with_runtime(|runtime| runtime.runtime_id().to_string())
    }

    pub fn app_id(&self) -> String {
        self.with_app_inner(|app| app.id().to_string())
    }

    pub fn is_taskbar(&self) -> bool {
        self.with_runtime(|runtime| runtime.presentation() == AppPresentation::Taskbar)
    }
}

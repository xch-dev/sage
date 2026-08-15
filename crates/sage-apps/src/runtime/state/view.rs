use serde::Serialize;
use specta::Type;

use crate::{
    AppPresentation, SageAppRuntimeMode, SageAppRuntimeVisibility, SageAppView, SharedRuntime,
};

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SageAppRuntimeRecordView {
    runtime_id: String,
    app: SageAppView,
    host_window_label: String,
    webview_label: String,
    presentation: AppPresentation,
    mode: SageAppRuntimeMode,
    visibility: SageAppRuntimeVisibility,
    started_at: i64,
    last_active_at: i64,
    internal: bool,
}

impl From<&SharedRuntime> for SageAppRuntimeRecordView {
    fn from(value: &SharedRuntime) -> Self {
        value.with_runtime(|runtime| Self {
            runtime_id: runtime.runtime_id(),
            app: runtime.app().into(),
            host_window_label: runtime.host_window_label(),
            webview_label: runtime.webview_label(),
            presentation: runtime.presentation(),
            mode: runtime.mode(),
            visibility: runtime.visibility(),
            started_at: runtime.started_at(),
            last_active_at: runtime.last_active_at(),
            internal: runtime.internal(),
        })
    }
}

impl From<SharedRuntime> for SageAppRuntimeRecordView {
    fn from(value: SharedRuntime) -> Self {
        Self::from(&value)
    }
}

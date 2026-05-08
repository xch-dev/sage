mod read;
mod remove;
mod types;
mod view;
mod write;

pub(crate) use types::{AppRuntimeState, SageAppRuntimeRecord, SharedRuntime};
pub use view::SageAppRuntimeRecordView;

pub use read::find_runtime_by_app_id_optional;

pub(crate) use read::{
    GetRuntimeError, find_active_taskbar_runtime, find_impostor_runtime_by_victim_app_id_optional,
    find_impostor_runtime_by_victim_app_id_optional_immediate,
    find_runtime_by_app_id_optional_immediate, find_runtime_by_runtime_id_optional,
    find_runtime_id_by_app_id_optional, get_runtime_by_app_id, list_runtimes,
};
pub(crate) use types::{
    ReadyToStopParams, RuntimeAckResult, SageAppRuntimeImpostorKind, SageAppRuntimeImpostorRecord,
    SageAppRuntimeMode, SageAppRuntimeVisibility, SetBeforeStopListenerParams,
    SharedImpostorRuntime,
};

pub(in crate::runtime) use read::is_apps_workspace_active;
pub(in crate::runtime) use write::{activate_apps_workspace, deactivate_apps_workspace};

pub(super) use remove::{
    remove_before_stop_listeners_by_app_id, remove_impostor_runtime_by_victim_app_id,
    remove_pending_stop_ready, remove_runtime_by_runtime_id, remove_runtime_id_by_app_id,
};
pub(super) use write::{write_impostor_runtime, write_pending_stop_ready, write_runtime};

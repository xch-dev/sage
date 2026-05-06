mod read;
mod remove;
mod types;
mod write;
mod view;

pub use view::SageAppRuntimeRecordView;
pub(crate) use types::{AppRuntimeState, SharedRuntime, SageAppRuntimeRecord};

pub use read::find_runtime_by_app_id_optional;

pub(crate) use read::{
    find_runtime_by_runtime_id_optional, find_runtime_id_by_app_id_optional, get_runtime_by_app_id,
    list_runtimes, find_impostor_runtime_by_victim_app_id_optional,
    find_impostor_runtime_by_victim_app_id_optional_immediate, find_runtime_by_app_id_optional_immediate, find_active_taskbar_runtime,
    GetRuntimeError
};
pub(crate) use types::{
    ReadyToStopParams, RuntimeAckResult, SetBeforeStopListenerParams,
    SageAppRuntimeMode, SageAppRuntimeVisibility, SageAppRuntimeImpostorKind, SharedImpostorRuntime,
    SageAppRuntimeImpostorRecord,
};

pub(super) use remove::{
    remove_before_stop_listeners_by_app_id, remove_pending_stop_ready,
    remove_runtime_by_runtime_id, remove_runtime_id_by_app_id, remove_impostor_runtime_by_victim_app_id,
};
pub(super) use write::{
    write_pending_stop_ready, write_runtime, write_impostor_runtime
};

mod bridge;
mod build;
mod capabilities;
mod db;
mod environment;
mod host;
mod lifecycle;
mod runtime;
mod sandbox;
mod security;
mod settings;
mod storage;
mod system_apps;
mod types;
mod utils;

// State
pub use db::AppsDb;
pub use host::AppsHostState;

// Commands
pub use bridge::{apps_invoke_bridge, apps_invoke_system_bridge};
pub use environment::apps_set_environment_theme;
pub use lifecycle::{
    apps_apply_app_update, apps_check_app_update, apps_clear_runtime_browsing_data,
    apps_list_installed_apps, apps_uninstall_app,
};
pub use runtime::{
    apps_clear_active_taskbar_runtime, apps_dev_reload_runtime, apps_enter_workspace,
    apps_focus_taskbar_runtime, apps_kill_taskbar_runtime, apps_leave_workspace,
    apps_list_runtimes, apps_start_system_app, apps_start_user_app,
};
pub use sandbox::{apps_get_app_launch_gate, apps_get_sandbox_state, apps_rerun_sandbox_tests};
pub use settings::{apps_get_auto_update_enabled, apps_set_auto_update_enabled};

// Bridge protocol
pub use security::{handle_system_app_protocol_request, handle_user_app_protocol_request};

// SDK types
pub use bridge::{export_system_bridge_typescript, export_user_bridge_typescript};

// Operations
pub use lifecycle::{process_pending_storage_cleanup, start_background_app_update_checker};
pub use runtime::process_sage_network_change;
pub use sandbox::ensure_initial_sandbox_run;

// Docs
pub use build::generate_docs;

// Internal
pub(crate) use bridge::*;
pub(crate) use capabilities::*;
pub(crate) use db::*;
pub(crate) use host::*;
pub(crate) use lifecycle::*;
pub(crate) use runtime::*;
pub(crate) use sandbox::*;
pub(crate) use security::*;
pub(crate) use storage::*;
pub(crate) use system_apps::*;
pub(crate) use types::*;
pub(crate) use utils::*;

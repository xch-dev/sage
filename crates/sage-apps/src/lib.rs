mod bridge;
mod build;
mod capabilities;
mod environment;
mod host;
mod lifecycle;
mod runtime;
mod sandbox;
mod security;
mod storage;
mod system_apps;
mod types;
mod utils;

// State
pub use host::AppsHostState;

// Commands
pub use bridge::commands::{
    apps_invoke_bridge, apps_invoke_system_bridge, get_user_capability_definitions,
};
pub use environment::commands::apps_set_environment_theme;
pub use lifecycle::{apps_clear_runtime_browsing_data, uninstall::uninstall_app};
pub use lifecycle::{
    install::commands::list_installed_apps,
    update::commands::{apply_app_update, check_app_update},
};
pub use runtime::commands::{
    apps_clear_active_taskbar_runtime, apps_create_inline_runtime, apps_dev_reload_runtime,
    apps_enter_workspace, apps_focus_taskbar_runtime, apps_kill_taskbar_runtime,
    apps_leave_workspace, apps_list_runtimes, apps_start_system_app,
};
pub use sandbox::commands::{
    apps_get_app_launch_gate, apps_get_sandbox_state, apps_rerun_sandbox_tests,
};

// Bridge
pub use security::{handle_system_app_protocol_request, handle_user_app_protocol_request};

// SDK
pub use bridge::ts_exports::{export_system_bridge_typescript, export_user_bridge_typescript};

// Operations
pub use lifecycle::process_pending_storage_cleanup;
pub use sandbox::runner::ensure_initial_sandbox_run;
pub use runtime::{process_sage_network_change};

// Docs
pub use build::docs::generate_docs;

mod bridge;
mod build;
mod capabilities;
mod host;
mod lifecycle;
mod runtime;
pub mod sandbox;
pub mod security;
pub mod storage;
pub mod system_apps;
pub mod types;
pub mod utils;
pub mod environment;

pub use bridge::{
    commands::{apps_invoke_bridge, apps_invoke_system_bridge, get_user_capability_definitions},
    ts_exports::{export_system_bridge_typescript, export_user_bridge_typescript}
};
pub use build::docs::generate_docs;
pub use host::AppsHostState;
pub use lifecycle::{
    install::commands::list_installed_apps,
    update::commands::{
        check_app_update,
        download_app_update,
        apply_app_update,
    },
    uninstall::uninstall_app,
    apps_clear_runtime_browsing_data, process_pending_storage_cleanup
};
pub use runtime::commands::{
    apps_start_system_app,
    apps_create_inline_runtime,
    apps_list_runtimes,
    apps_focus_taskbar_runtime,
    apps_clear_active_taskbar_runtime,
    apps_kill_taskbar_runtime,
    apps_dev_reload_runtime
};
pub use security::{handle_system_app_protocol_request, handle_user_app_protocol_request};


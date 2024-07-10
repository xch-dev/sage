use app_state::AppStateInner;
use tauri::Manager;
use tokio::sync::Mutex;

mod app_state;
mod commands;
mod config;
mod error;
mod models;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .invoke_handler(tauri::generate_handler![
            commands::generate_mnemonic,
            commands::active_wallet,
            commands::wallet_config,
            commands::login_wallet,
            commands::logout_wallet,
            commands::wallet_list,
            commands::create_wallet,
            commands::import_wallet,
            commands::delete_wallet,
            commands::rename_wallet,
            commands::set_derivation_mode,
            commands::set_derivation_batch_size,
        ])
        .setup(|app| {
            let path = app.path().app_data_dir()?;
            let state = AppStateInner::new(path);
            state.initialize()?;
            app.manage(Mutex::new(state));
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

use app_state::AppStateInner;
use tauri::Manager;
use tokio::sync::Mutex;

mod app_state;
mod commands;
mod config;
mod error;
mod models;
mod peer_discovery;
mod sync_manager;
mod wallet;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .invoke_handler(tauri::generate_handler![
            // Network Config
            commands::network_config,
            commands::set_peer_mode,
            commands::set_target_peers,
            commands::set_network_id,
            // Wallet Config
            commands::wallet_config,
            commands::set_derivation_mode,
            commands::set_derivation_batch_size,
            commands::set_derivation_index,
            // Networks
            commands::network_list,
            // Keychain
            commands::active_wallet,
            commands::wallet_list,
            commands::login_wallet,
            commands::logout_wallet,
            commands::generate_mnemonic,
            commands::create_wallet,
            commands::import_wallet,
            commands::delete_wallet,
            commands::rename_wallet,
            // Setup
            commands::initialize,
            // Wallet
            commands::sync_info,
            // Peers
            commands::peer_list,
        ])
        .setup(|app| {
            let path = app.path().app_data_dir()?;
            let state = AppStateInner::new(app.handle().clone(), &path);
            app.manage(Mutex::new(state));
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

use app_state::AppStateInner;
use models::SyncEvent;
use specta_typescript::Typescript;
use tauri::Manager;
use tauri_specta::{collect_commands, collect_events, Builder};
use tokio::sync::Mutex;

mod app_state;
mod commands;
mod error;
mod models;
mod utils;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = Builder::<tauri::Wry>::new()
        // Then register them (separated by a comma)
        .commands(collect_commands![
            // Network Config
            commands::network_config,
            commands::set_discover_peers,
            commands::set_target_peers,
            commands::set_network_id,
            // Wallet Config
            commands::wallet_config,
            commands::set_derive_automatically,
            commands::set_derivation_batch_size,
            // Networks
            commands::network_list,
            // Keychain
            commands::active_wallet,
            commands::get_wallet_secrets,
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
            commands::get_sync_status,
            commands::get_coins,
            commands::get_cats,
            commands::get_dids,
            commands::get_nfts,
            commands::validate_address,
            // Transactions
            commands::send,
            // Peers
            commands::get_peers,
            commands::remove_peer,
        ])
        .events(collect_events![SyncEvent]);

    #[cfg(debug_assertions)] // <- Only export on non-release builds
    builder
        .export(Typescript::default(), "../src/bindings.ts")
        .expect("Failed to export TypeScript bindings");

    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            builder.mount_events(app);
            let app_handle = app.handle().clone();
            let path = app.path().app_data_dir()?;
            let state = AppStateInner::new(app_handle, &path);
            app.manage(Mutex::new(state));
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

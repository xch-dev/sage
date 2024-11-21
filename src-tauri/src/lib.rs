use app_state::AppStateInner;
use sage_api::SyncEvent;
use tauri::Manager;
use tauri_specta::{collect_commands, collect_events, Builder};
use tokio::sync::Mutex;

mod app_state;
mod commands;
mod error;

#[cfg(all(debug_assertions, not(mobile)))]
use specta_typescript::{BigIntExportBehavior, Typescript};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = Builder::<tauri::Wry>::new()
        // Then register them (separated by a comma)
        .commands(collect_commands![
            commands::initialize,
            commands::login,
            commands::logout,
            commands::resync,
            commands::generate_mnemonic,
            commands::import_key,
            commands::delete_key,
            commands::rename_key,
            commands::get_keys,
            commands::get_key,
            commands::get_secret_key,
            commands::send_xch,
            commands::combine_xch,
            commands::split_xch,
            commands::send_cat,
            commands::combine_cat,
            commands::split_cat,
            commands::issue_cat,
            commands::create_did,
            commands::bulk_mint_nfts,
            commands::transfer_nfts,
            commands::transfer_dids,
            commands::add_nft_uri,
            commands::assign_nfts_to_did,
            commands::sign_coin_spends,
            commands::submit_transaction,
            commands::get_sync_status,
            commands::get_addresses,
            commands::get_xch_coins,
            commands::get_cat_coins,
            commands::get_cats,
            commands::get_cat,
            commands::get_dids,
            commands::get_nft_status,
            commands::get_nft_collections,
            commands::get_nft_collection,
            commands::get_nfts,
            commands::get_nft,
            commands::get_pending_transactions,
            commands::validate_address,
            commands::make_offer,
            commands::take_offer,
            commands::view_offer,
            commands::network_config,
            commands::set_discover_peers,
            commands::set_target_peers,
            commands::set_network_id,
            commands::wallet_config,
            commands::set_derive_automatically,
            commands::set_derivation_batch_size,
            commands::get_networks,
            commands::update_cat,
            commands::remove_cat,
            commands::update_did,
            commands::update_nft,
            commands::get_peers,
            commands::add_peer,
            commands::remove_peer,
        ])
        .events(collect_events![SyncEvent]);

    // On mobile or release mode we should not export the TypeScript bindings
    #[cfg(all(debug_assertions, not(mobile)))]
    builder
        .export(
            Typescript::default().bigint(BigIntExportBehavior::Number),
            "../src/bindings.ts",
        )
        .expect("Failed to export TypeScript bindings");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
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

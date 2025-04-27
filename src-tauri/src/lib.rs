use app_state::{AppState, Initialized, RpcTask};
use rustls::crypto::aws_lc_rs::default_provider;
use sage::Sage;
use sage_api::SyncEvent;
use tauri::Manager;
use tauri_specta::{collect_commands, collect_events, Builder, ErrorHandlingMode};
use tokio::sync::Mutex;

mod app_state;
mod commands;
mod error;

#[cfg(all(debug_assertions, not(mobile)))]
use specta_typescript::{BigIntExportBehavior, Typescript};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    default_provider()
        .install_default()
        .expect("could not install AWS LC provider");

    let builder = Builder::<tauri::Wry>::new()
        .error_handling(ErrorHandlingMode::Throw)
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
            commands::bulk_send_xch,
            commands::combine_xch,
            commands::auto_combine_xch,
            commands::split_xch,
            commands::send_cat,
            commands::bulk_send_cat,
            commands::combine_cat,
            commands::auto_combine_cat,
            commands::split_cat,
            commands::issue_cat,
            commands::create_did,
            commands::bulk_mint_nfts,
            commands::transfer_nfts,
            commands::transfer_dids,
            commands::normalize_dids,
            commands::add_nft_uri,
            commands::assign_nfts_to_did,
            commands::sign_coin_spends,
            commands::view_coin_spends,
            commands::submit_transaction,
            commands::get_sync_status,
            commands::check_address,
            commands::get_derivations,
            commands::get_are_coins_spendable,
            commands::get_spendable_coin_count,
            commands::get_coins_by_ids,
            commands::get_xch_coins,
            commands::get_cat_coins,
            commands::get_cats,
            commands::get_cat,
            commands::get_dids,
            commands::get_minter_did_ids,
            commands::get_nft_collections,
            commands::get_nft_collection,
            commands::get_nfts,
            commands::get_nft,
            commands::get_nft_data,
            commands::get_nft_icon,
            commands::get_nft_thumbnail,
            commands::get_pending_transactions,
            commands::get_transactions,
            commands::validate_address,
            commands::make_offer,
            commands::take_offer,
            commands::combine_offers,
            commands::view_offer,
            commands::import_offer,
            commands::get_offers,
            commands::get_offer,
            commands::delete_offer,
            commands::cancel_offer,
            commands::network_config,
            commands::set_discover_peers,
            commands::set_target_peers,
            commands::set_network,
            commands::set_network_override,
            commands::wallet_config,
            commands::get_networks,
            commands::get_network,
            commands::update_cat,
            commands::remove_cat,
            commands::update_did,
            commands::update_nft,
            commands::update_nft_collection,
            commands::redownload_nft,
            commands::increase_derivation_index,
            commands::get_peers,
            commands::add_peer,
            commands::remove_peer,
            commands::filter_unlocked_coins,
            commands::get_asset_coins,
            commands::sign_message_with_public_key,
            commands::sign_message_by_address,
            commands::send_transaction_immediately,
            commands::is_rpc_running,
            commands::start_rpc_server,
            commands::stop_rpc_server,
            commands::get_rpc_run_on_startup,
            commands::set_rpc_run_on_startup,
            commands::move_key,
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

    let mut tauri_builder = tauri::Builder::default();

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        tauri_builder = tauri_builder.plugin(tauri_plugin_window_state::Builder::new().build());
    }

    tauri_builder
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_os::init())
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            #[cfg(mobile)]
            {
                app.handle().plugin(tauri_plugin_barcode_scanner::init())?;
            }

            #[cfg(mobile)]
            {
                app.handle().plugin(tauri_plugin_safe_area_insets::init())?;
            }
            builder.mount_events(app);
            let path = app.path().app_data_dir()?;
            let app_state = AppState::new(Mutex::new(Sage::new(&path)));
            app.manage(Initialized(Mutex::new(false)));
            app.manage(RpcTask(Mutex::new(None)));
            app.manage(app_state);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

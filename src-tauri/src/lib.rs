use app_state::{AppState, Initialized, RpcTask};
use rustls::crypto::aws_lc_rs::default_provider;
use sage::Sage;
use sage_api::SyncEvent;
use sage_apps as apps;
use tauri::Manager;
use tauri_specta::{Builder, ErrorHandlingMode, collect_commands, collect_events};
use tokio::sync::Mutex;

mod app_state;
mod commands;
mod error;

use sage_apps::bridge::RustBridgeApprovalEvent;
use sage_apps::security::handle_user_app_protocol_request;
#[cfg(all(debug_assertions, not(mobile)))]
use specta_typescript::{BigIntExportBehavior, Typescript};
use sage_apps::{handle_system_app_protocol_request, AppsHostState};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    default_provider()
        .install_default()
        .expect("could not install AWS LC provider");

    let builder = Builder::<tauri::Wry>::new()
        .error_handling(ErrorHandlingMode::Throw)
        .commands(collect_commands![
            commands::initialize,
            commands::login,
            commands::logout,
            commands::resync,
            commands::generate_mnemonic,
            commands::import_key,
            commands::delete_key,
            commands::delete_database,
            commands::rename_key,
            commands::get_keys,
            commands::set_wallet_emoji,
            commands::get_key,
            commands::get_secret_key,
            commands::send_xch,
            commands::bulk_send_xch,
            commands::combine,
            commands::split,
            commands::auto_combine_xch,
            commands::send_cat,
            commands::bulk_send_cat,
            commands::auto_combine_cat,
            commands::issue_cat,
            commands::create_did,
            commands::bulk_mint_nfts,
            commands::transfer_nfts,
            commands::transfer_dids,
            commands::normalize_dids,
            commands::mint_option,
            commands::transfer_options,
            commands::exercise_options,
            commands::add_nft_uri,
            commands::assign_nfts_to_did,
            commands::finalize_clawback,
            commands::create_transaction,
            commands::sign_coin_spends,
            commands::view_coin_spends,
            commands::submit_transaction,
            commands::get_sync_status,
            commands::get_version,
            commands::get_database_stats,
            commands::perform_database_maintenance,
            commands::check_address,
            commands::get_derivations,
            commands::get_are_coins_spendable,
            commands::get_spendable_coin_count,
            commands::get_coins_by_ids,
            commands::get_coins,
            commands::get_cats,
            commands::get_all_cats,
            commands::get_token,
            commands::get_dids,
            commands::get_minter_did_ids,
            commands::get_options,
            commands::get_option,
            commands::get_nft_collections,
            commands::get_nft_collection,
            commands::get_nfts,
            commands::get_nft,
            commands::get_nft_data,
            commands::get_nft_icon,
            commands::get_nft_thumbnail,
            commands::get_pending_transactions,
            commands::get_transaction,
            commands::get_transactions,
            commands::validate_address,
            commands::make_offer,
            commands::take_offer,
            commands::combine_offers,
            commands::view_offer,
            commands::import_offer,
            commands::get_offers,
            commands::get_offers_for_asset,
            commands::get_offer,
            commands::delete_offer,
            commands::cancel_offer,
            commands::cancel_offers,
            commands::network_config,
            commands::set_discover_peers,
            commands::set_target_peers,
            commands::set_network,
            commands::set_network_override,
            commands::wallet_config,
            commands::default_wallet_config,
            commands::get_networks,
            commands::get_network,
            commands::set_delta_sync,
            commands::set_delta_sync_override,
            commands::set_change_address,
            commands::update_cat,
            commands::resync_cat,
            commands::update_did,
            commands::update_option,
            commands::update_nft,
            commands::update_nft_collection,
            commands::redownload_nft,
            commands::increase_derivation_index,
            commands::get_peers,
            commands::get_user_theme,
            commands::get_user_themes,
            commands::save_user_theme,
            commands::delete_user_theme,
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
            commands::switch_wallet,
            commands::move_key,
            commands::download_cni_offercode,
            commands::get_logs,
            commands::is_asset_owned,
            commands::get_xch_usd_price,
            apps::bridge::commands::apps_invoke_bridge,
            apps::bridge::commands::apps_invoke_system_bridge,
            apps::bridge::commands::apps_resolve_bridge_approval,
            apps::bridge::commands::get_user_capability_definitions,
            apps::sandbox::commands::apps_get_sandbox_state,
            apps::sandbox::commands::apps_get_app_launch_gate,
            apps::sandbox::commands::apps_rerun_sandbox_tests,
            apps::lifecycle::install::commands::list_installed_apps,
            apps::lifecycle::install::commands::preview_app_zip,
            apps::lifecycle::install::commands::preview_app_url,
            apps::lifecycle::install::commands::install_app_zip,
            apps::lifecycle::install::commands::install_app_url,
            apps::lifecycle::uninstall::uninstall_app,
            apps::lifecycle::update::commands::check_app_update,
            apps::lifecycle::update::commands::download_app_update,
            apps::lifecycle::update::commands::apply_app_update,
            apps::lifecycle::update::commands::apps_update_permissions,
            apps::lifecycle::apps_clear_runtime_browsing_data,
            apps::sandbox::commands::get_builtin_test_app,
            apps::system_apps::get_builtin_system_app,
            apps::runtime::commands::apps_create_inline_runtime,
            apps::runtime::commands::apps_list_runtimes,
            apps::runtime::commands::apps_focus_runtime,
            apps::runtime::commands::apps_hide_runtime,
            apps::runtime::commands::apps_kill_runtime,
        ])
        .events(collect_events![SyncEvent, RustBridgeApprovalEvent]);

    #[cfg(all(debug_assertions, not(mobile)))]
    builder
        .export(
            Typescript::default().bigint(BigIntExportBehavior::Number),
            "../src/bindings.ts",
        )
        .expect("Failed to export TypeScript bindings");

    let mut tauri_builder = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_os::init());

    #[cfg(not(mobile))]
    {
        tauri_builder = tauri_builder
            .plugin(tauri_plugin_window_state::Builder::new().build())
            .plugin(tauri_plugin_fs::init())
            .plugin(tauri_plugin_dialog::init());
    }

    #[cfg(mobile)]
    {
        tauri_builder = tauri_builder
            .plugin(tauri_plugin_barcode_scanner::init())
            .plugin(tauri_plugin_safe_area_insets::init())
            .plugin(tauri_plugin_biometric::init())
            .plugin(tauri_plugin_sharesheet::init())
            .plugin(tauri_plugin_sage::init());
    }

    tauri_builder
        .register_uri_scheme_protocol("sage-app", move |ctx, request| {
            tauri::async_runtime::block_on(
                handle_user_app_protocol_request(&ctx, &request)
            )
        })
        .register_uri_scheme_protocol("sage-system-app", move |ctx, request| {
            tauri::async_runtime::block_on(
                handle_system_app_protocol_request(&ctx, &request)
            )
        })
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            builder.mount_events(app);
            let path = app.path().app_data_dir()?;
            let app_state = AppState::new(Mutex::new(Sage::new(&path, false)));
            app.manage(Initialized(Mutex::new(false)));
            app.manage(RpcTask(Mutex::new(None)));
            app.manage(app_state);
            app.manage(AppsHostState::default());

            let app_handle = app.handle().clone();
            let cleanup_base_path = path.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(err) = apps::lifecycle::process_pending_storage_cleanup(
                    &app_handle,
                    &cleanup_base_path,
                )
                .await
                {
                    eprintln!("failed to retry pending storage cleanup on startup: {err}");
                }
            });

            let sandbox_app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(err) =
                    apps::sandbox::runner::ensure_initial_sandbox_run(sandbox_app_handle).await
                {
                    eprintln!("failed to start initial sandbox run: {err}");
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

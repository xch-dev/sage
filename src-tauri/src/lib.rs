use std::path::Path;
#[cfg(all(debug_assertions, not(mobile)))]
use std::path::PathBuf;

use app_state::{AppState, Initialized, RpcTask};
use rustls::crypto::aws_lc_rs::default_provider;
use sage::Sage;
use sage_api::SyncEvent;
use tauri::Manager;
#[cfg(all(not(mobile), not(debug_assertions)))]
use tauri::path::BaseDirectory;
use tauri_specta::{Builder, ErrorHandlingMode, collect_commands, collect_events};
use tokio::sync::Mutex;

mod app_state;
mod commands;
mod error;

#[cfg(not(mobile))]
use sage_apps as apps;

#[cfg(not(mobile))]
use sage_apps::{
    AppsHostState, handle_system_app_protocol_request, handle_user_app_protocol_request,
};

#[cfg(all(debug_assertions, not(mobile)))]
use specta_typescript::{BigIntExportBehavior, Typescript};

macro_rules! sage_commands {
    ($($extra_command:tt)*) => {
        collect_commands![
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
            commands::get_wallet_address,
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
            $($extra_command)*
        ]
    };
}

#[cfg(not(mobile))]
fn specta_builder() -> Builder<tauri::Wry> {
    Builder::<tauri::Wry>::new()
        .error_handling(ErrorHandlingMode::Throw)
        .commands(sage_commands![
            apps::apps_enter_workspace,
            apps::apps_leave_workspace,
            apps::apps_invoke_bridge,
            apps::apps_invoke_system_bridge,
            apps::apps_set_environment_theme,
            apps::apps_get_sandbox_state,
            apps::apps_get_app_launch_gate,
            apps::apps_rerun_sandbox_tests,
            apps::apps_list_installed_apps,
            apps::apps_uninstall_app,
            apps::apps_check_app_update,
            apps::apps_apply_app_update,
            apps::apps_recover_app_update,
            apps::apps_clear_runtime_browsing_data,
            apps::apps_start_system_app,
            apps::apps_start_user_app,
            apps::apps_list_runtimes,
            apps::apps_focus_taskbar_runtime,
            apps::apps_clear_active_taskbar_runtime,
            apps::apps_kill_taskbar_runtime,
            apps::apps_reorder_taskbar_runtimes,
            apps::apps_dev_reload_runtime,
            apps::apps_get_auto_update_enabled,
            apps::apps_set_auto_update_enabled,
        ])
        .events(collect_events![SyncEvent])
}

#[cfg(all(debug_assertions, not(mobile)))]
pub fn export_bindings() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../src/bindings.ts");

    specta_builder()
        .export(
            Typescript::default().bigint(BigIntExportBehavior::Number),
            path,
        )
        .expect("Failed to export TypeScript bindings");
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    default_provider()
        .install_default()
        .expect("could not install AWS LC provider");

    #[cfg(not(mobile))]
    let builder = specta_builder();

    #[cfg(mobile)]
    let builder = Builder::<tauri::Wry>::new()
        .error_handling(ErrorHandlingMode::Throw)
        .commands(sage_commands![])
        .events(collect_events![SyncEvent]);

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

    #[cfg(not(mobile))]
    {
        tauri_builder = tauri_builder
            .register_asynchronous_uri_scheme_protocol(
                "sage-app",
                move |ctx, request, responder| {
                    let app_handle = ctx.app_handle().clone();
                    let webview_label = ctx.webview_label().to_string();

                    tauri::async_runtime::spawn(async move {
                        let response =
                            handle_user_app_protocol_request(app_handle, webview_label, request)
                                .await;

                        responder.respond(response);
                    });
                },
            )
            .register_asynchronous_uri_scheme_protocol(
                "sage-system-app",
                move |ctx, request, responder| {
                    let app_handle = ctx.app_handle().clone();
                    let webview_label = ctx.webview_label().to_string();

                    tauri::async_runtime::spawn(async move {
                        let response =
                            handle_system_app_protocol_request(app_handle, webview_label, request)
                                .await;

                        responder.respond(response);
                    });
                },
            );
    }

    tauri_builder
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            builder.mount_events(app);

            let root_override = sage_config::sage_root_override();

            create_main_window(app, root_override.as_deref())?;

            let path = match root_override {
                Some(path) => path,
                None => app.path().app_data_dir()?,
            };
            let app_state = AppState::new(Mutex::new(Sage::new(&path, false)));

            app.manage(Initialized(Mutex::new(false)));
            app.manage(RpcTask(Mutex::new(None)));
            app.manage(app_state);

            #[cfg(not(mobile))]
            {
                #[cfg(not(debug_assertions))]
                {
                    let bundled_builtin_apps = app
                        .path()
                        .resolve("builtin-apps", BaseDirectory::Resource)?;

                    if bundled_builtin_apps.is_dir() {
                        apps::set_builtin_apps_root(bundled_builtin_apps);
                    } else {
                        tracing::warn!(
                            "bundled builtin apps directory not found at {}; using development path",
                            bundled_builtin_apps.display()
                        );
                    }
                }

                let apps_db = tauri::async_runtime::block_on(apps::AppsDb::initialize(&path))
                    .expect("failed to initialize Sage apps database");

                app.manage(AppsHostState::new(path.clone(), apps_db));

                apps::start_background_app_update_checker(app.handle().clone());

                let app_handle = app.handle().clone();

                tauri::async_runtime::spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                    tracing::info!("starting pending storage cleanup task");

                    if let Err(err) = apps::process_pending_storage_cleanup(&app_handle).await {
                        tracing::error!(
                            "failed to retry pending storage cleanup on startup: {err}"
                        );
                    }

                    tracing::info!("pending storage cleanup task finished");
                });
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Creates the main window from `tauri.conf.json`, where it is declared with `"create": false`.
///
/// When `SAGE_ROOT` is set, the main webview gets its own storage so that frontend state such as
/// local storage isn't shared with instances using a different root.
fn create_main_window(app: &tauri::App, isolated_root: Option<&Path>) -> tauri::Result<()> {
    let config = app
        .config()
        .app
        .windows
        .first()
        .ok_or_else(|| anyhow::anyhow!("missing main window config"))?;

    #[allow(unused_mut)]
    let mut builder = tauri::WebviewWindowBuilder::from_config(app.handle(), config)?;

    if let Some(root) = isolated_root {
        #[cfg(any(target_os = "windows", target_os = "linux"))]
        {
            builder = builder.data_directory(root.join("webview"));
        }

        #[cfg(target_os = "macos")]
        {
            builder = builder.data_store_identifier(data_store_identifier_for(root));
        }

        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        let _ = root;
    }

    builder.build()?;

    Ok(())
}

#[cfg(target_os = "macos")]
fn data_store_identifier_for(root: &Path) -> [u8; 16] {
    use chia_wallet_sdk::chia::sha2::Sha256;

    let mut hasher = Sha256::new();
    hasher.update(b"sage-main-webview:");
    hasher.update(root.as_os_str().as_encoded_bytes());

    let hash = hasher.finalize();
    let mut identifier = [0; 16];
    identifier.copy_from_slice(&hash[..16]);
    identifier
}

#[cfg(test)]
mod tests {
    #[test]
    fn tauri_identifier_matches_default_sage_root() {
        let config: serde_json::Value =
            serde_json::from_str(include_str!("../tauri.conf.json")).unwrap();

        assert_eq!(
            config["identifier"].as_str(),
            Some(sage_config::SAGE_APP_IDENTIFIER),
            "the CLI and RPC client resolve the default root from this identifier"
        );
    }
}

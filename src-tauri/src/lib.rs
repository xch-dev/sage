use std::{
    fs,
    path::{Path, PathBuf},
};

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
            commands::quit,
            commands::login,
            commands::logout,
            commands::resync,
            commands::generate_mnemonic,
            commands::import_wallet,
            commands::import_watch_wallet,
            commands::add_watch_addresses,
            commands::remove_watch_addresses,
            commands::import_vault,
            commands::get_vault,
            commands::attach_vault_signer,
            commands::rebuild_vault_config,
            commands::find_vault,
            commands::delete_wallet,
            commands::delete_database,
            commands::rename_wallet,
            commands::set_wallet_emoji,
            commands::get_wallet,
            commands::get_wallets,
            commands::get_wallet_address,
            commands::get_key,
            commands::get_keys,
            commands::add_key,
            commands::delete_key,
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
            commands::claim_clawback,
            commands::create_transaction,
            commands::sign_coin_spends,
            commands::view_coin_spends,
            commands::submit_transaction,
            commands::get_sync_status,
            commands::get_version,
            commands::get_database_stats,
            commands::perform_database_maintenance,
            commands::check_address,
            commands::resolve_handle,
            commands::get_handle,
            commands::get_handles,
            commands::lookup_address_handles,
            commands::get_contacts,
            commands::get_contact,
            commands::add_contact,
            commands::update_contact,
            commands::delete_contact,
            commands::lookup_address_contacts,
            commands::export_contacts,
            commands::import_contacts,
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
            commands::get_nft_metadata_updater,
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
            commands::get_selected_theme,
            commands::set_selected_theme,
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

            let default_root = app.path().app_data_dir()?;
            let root_override = resolve_root_override(&default_root)?;

            // This is also Tauri's default data directory for webviews on Windows and Linux.
            let webview_root = match &root_override {
                Some(root) => root.join("webview"),
                None => app.path().app_local_data_dir()?,
            };

            // WebKit data stores are global on macOS, so a custom root needs its own.
            let data_store_identifier = if cfg!(target_os = "macos") && root_override.is_some() {
                Some(load_data_store_identifier(&webview_root)?)
            } else {
                None
            };

            // Without data store identifiers, WebKit silently falls back to the default data
            // store, which is shared with the default root. Keep webview storage in memory instead.
            #[cfg(target_os = "macos")]
            let in_memory_webview_storage =
                data_store_identifier.is_some() && !supports_data_store_identifiers();
            #[cfg(not(target_os = "macos"))]
            let in_memory_webview_storage = false;

            if in_memory_webview_storage {
                tracing::warn!(
                    "macOS 14 or later is required to persist webview storage with {}",
                    sage_config::SAGE_ROOT_ENV
                );
            }

            create_main_window(
                app,
                &webview_root,
                data_store_identifier,
                in_memory_webview_storage,
            )?;

            let path = root_override.unwrap_or(default_root);
            let app_state = AppState::new(Mutex::new(Sage::new(&path, false)));

            app.manage(Initialized(Mutex::new(None)));
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

                app.manage(
                    AppsHostState::new(path.clone(), webview_root, apps_db)
                        .with_builtin_storage_namespace(
                            data_store_identifier
                                .map(|identifier| uuid::Uuid::from_bytes(identifier).to_string()),
                        )
                        .with_in_memory_webview_storage(in_memory_webview_storage),
                );

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

/// Returns the root set by `SAGE_ROOT`, unless it's the default root.
///
/// `SAGE_ROOT` is ignored on mobile, where each app has a single sandboxed data directory.
fn resolve_root_override(default_root: &Path) -> anyhow::Result<Option<PathBuf>> {
    if cfg!(mobile) {
        return Ok(None);
    }

    let Some(root) = sage_config::sage_root_override() else {
        return Ok(None);
    };

    // The root has to exist to be compared reliably with the default root.
    fs::create_dir_all(&root).map_err(|err| {
        anyhow::anyhow!(
            "failed to create {} directory {}: {err}",
            sage_config::SAGE_ROOT_ENV,
            root.display()
        )
    })?;

    // Pointing `SAGE_ROOT` at the default root shouldn't hide the main webview's existing
    // storage, such as WalletConnect sessions and settings.
    if is_same_path(&root, default_root) {
        return Ok(None);
    }

    Ok(Some(root))
}

/// Creates the main window from `tauri.conf.json`, where it is declared with `"create": false`.
///
/// When `SAGE_ROOT` is set, the main webview gets its own storage so that frontend state, such as
/// `WalletConnect` sessions and settings, isn't shared with instances using a different root.
fn create_main_window(
    app: &tauri::App,
    webview_root: &Path,
    data_store_identifier: Option<[u8; 16]>,
    in_memory_storage: bool,
) -> anyhow::Result<()> {
    let config = app
        .config()
        .app
        .windows
        .first()
        .ok_or_else(|| anyhow::anyhow!("missing main window config"))?;

    #[allow(unused_mut)]
    let mut builder = tauri::WebviewWindowBuilder::from_config(app.handle(), config)?;

    // Set explicitly, since app webviews share or nest inside this directory.
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    {
        builder = builder.data_directory(webview_root.to_path_buf());
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    let _ = webview_root;

    if in_memory_storage {
        builder = builder.incognito(true);
    } else {
        #[cfg(target_os = "macos")]
        if let Some(identifier) = data_store_identifier {
            builder = builder.data_store_identifier(identifier);
        }

        #[cfg(not(target_os = "macos"))]
        let _ = data_store_identifier;
    }

    builder.build()?;

    Ok(())
}

fn is_same_path(a: &Path, b: &Path) -> bool {
    match (a.canonicalize(), b.canonicalize()) {
        (Ok(a), Ok(b)) => a == b,
        _ => a == b,
    }
}

#[cfg(target_os = "macos")]
fn supports_data_store_identifiers() -> bool {
    // Treat an unknown version as unsupported, so storage is never shared by mistake.
    matches!(
        tauri_plugin_os::version(),
        tauri_plugin_os::Version::Semantic(major, _, _) if major >= 14
    )
}

/// Loads the identifier of the main webview's data store, generating it on first use.
///
/// The identifier is random and stored inside the root, so deleting the root also abandons its
/// webview storage, and different spellings of the same root path share it.
fn load_data_store_identifier(webview_root: &Path) -> anyhow::Result<[u8; 16]> {
    let path = webview_root.join("data-store-id");

    match fs::read_to_string(&path) {
        Ok(contents) => match uuid::Uuid::parse_str(contents.trim()) {
            Ok(identifier) => return Ok(*identifier.as_bytes()),
            Err(err) => tracing::warn!(
                "replacing invalid webview data store identifier in {}: {err}",
                path.display()
            ),
        },
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
        Err(err) => anyhow::bail!("failed to read {}: {err}", path.display()),
    }

    let identifier = uuid::Uuid::new_v4();

    fs::create_dir_all(webview_root)
        .map_err(|err| anyhow::anyhow!("failed to create {}: {err}", webview_root.display()))?;

    let temp_path = path.with_extension("tmp");
    fs::write(&temp_path, identifier.to_string())
        .and_then(|()| fs::rename(&temp_path, &path))
        .map_err(|err| anyhow::anyhow!("failed to write {}: {err}", path.display()))?;

    Ok(*identifier.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_path_resolves_equivalent_paths() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("root");
        std::fs::create_dir(&root).unwrap();

        assert!(is_same_path(&root, &dir.path().join("root/")));
        assert!(is_same_path(&root, &root.join("../root")));
        assert!(!is_same_path(&root, dir.path()));
        assert!(is_same_path(
            &dir.path().join("missing"),
            &dir.path().join("missing/")
        ));
    }

    #[test]
    fn data_store_identifier_is_persisted() {
        let dir = tempfile::tempdir().unwrap();
        let webview_root = dir.path().join("webview");

        let identifier = load_data_store_identifier(&webview_root).unwrap();

        assert_eq!(
            load_data_store_identifier(&webview_root).unwrap(),
            identifier
        );
        assert_ne!(
            load_data_store_identifier(&dir.path().join("other")).unwrap(),
            identifier
        );
    }

    #[test]
    fn invalid_data_store_identifier_is_replaced() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("data-store-id"), "invalid").unwrap();

        let identifier = load_data_store_identifier(dir.path()).unwrap();

        assert_eq!(load_data_store_identifier(dir.path()).unwrap(), identifier);
    }

    /// The commands a permission file allows.
    fn allowed_commands(permissions: &str) -> Vec<String> {
        let permissions: toml::Value = toml::from_str(permissions).unwrap();

        permissions["permission"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|permission| permission["commands"]["allow"].as_array().unwrap())
            .map(|command| command.as_str().unwrap().to_string())
            .collect()
    }

    /// Webviews can only call commands that a permission allows, and a command they aren't
    /// allowed to call fails without an error Sage can show. So every registered command has to
    /// be allowed for the main webview, or for app webviews in `bridge.toml`, and the names the
    /// main webview is allowed to call have to still exist.
    #[test]
    fn permissions_allow_every_command() {
        let commands: Vec<&str> = include_str!("lib.rs")
            .lines()
            .map(str::trim)
            .filter_map(|line| {
                line.strip_prefix("commands::")
                    .or_else(|| line.strip_prefix("apps::"))
            })
            .filter_map(|line| line.strip_suffix(','))
            .collect();

        let main_host = allowed_commands(include_str!("../permissions/main-host.toml"));
        let bridge = allowed_commands(include_str!("../permissions/bridge.toml"));

        assert!(commands.contains(&"get_wallets"));
        assert!(commands.contains(&"apps_list_installed_apps"));

        for command in &commands {
            assert!(
                main_host
                    .iter()
                    .chain(&bridge)
                    .any(|allowed| allowed == command),
                "`{command}` isn't allowed in any permission file"
            );
        }

        for name in &main_host {
            assert!(
                commands.contains(&name.as_str()),
                "`{name}` in main-host.toml isn't a command"
            );
        }
    }

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

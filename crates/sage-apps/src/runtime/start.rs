use std::collections::BTreeMap;

use specta::Type;
use tauri::webview::NewWindowResponse;
use tauri::{AppHandle, LogicalPosition, LogicalSize, State, WebviewBuilder, WebviewUrl, Wry};

use crate::lifecycle::write_installed_app_metadata;
use crate::runtime::state::{SageAppRuntimeRecord, write_runtime, remove_runtime_by_runtime_id, remove_runtime_id_by_app_id, remove_impostor_runtime_by_victim_app_id, write_impostor_runtime};
use crate::runtime::webview_locator::{
    get_sage_window, get_webview_in_sage_window,
};
use crate::runtime::{build_entry_src, build_entry_src_for, emit_runtime_manager_runtimes_changed, is_allowed_app_url, resolve_app, SageAppRuntimeImpostorKind, SageAppRuntimeImpostorRecord, SageAppRuntimeMode, SageAppRuntimeVisibility, SharedImpostorRuntime, SharedRuntime};
use crate::storage::parse_data_store_id;
use crate::types::{AppPresentation, InstalledSageAppStorage, ResolvedApp, ResolvedStoppedApp, SharedSageApp};
use crate::{AppsHostState, sandbox};

#[derive(Debug, Type)]
#[serde(rename_all = "camelCase")]
pub struct CreateRuntimeArgs {
    pub app_id: String,
    pub presentation: AppPresentation,
    pub mode: SageAppRuntimeMode,
    pub visibility: SageAppRuntimeVisibility,
    pub debug_layout: bool,
    pub query: BTreeMap<String, String>,
}

pub(in crate::runtime) struct CreateImpostorRuntimeArgs {
    pub kind: SageAppRuntimeImpostorKind,
    pub visibility: SageAppRuntimeVisibility,
    pub debug_layout: bool,
    pub query: BTreeMap<String, String>,
}

pub async fn create_runtime(
    app_handle: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    args: CreateRuntimeArgs,
) -> Result<SharedRuntime, String> {
    let app = match resolve_app(app_handle, &args.app_id).await.map_err(|e| e.to_string())? {
        ResolvedApp::Running(running) => return Ok(running.runtime()),
        ResolvedApp::Stopped(stopped) => stopped.into_app()
    };

    let is_internal = app.with(|app| app.common().is_sandbox_test());
    if !is_internal {
        check_gates(apps_state, &app).await?;
    }

    app.taint_storage_if_runtime_can_persist_secrets();
    persist_runtime_side_effects(&app)?;

    let sage_window = get_sage_window(app_handle)?;
    let webview_label = app.webview_label();
    let runtime = SageAppRuntimeRecord::new(
        &app,
        sage_window.label(),
        &webview_label,
        args.presentation,
        SageAppRuntimeMode::Inline,
        args.visibility,
        is_internal,
    ).map_err(|err| err.to_string())?;
    let shared_runtime = write_runtime(apps_state, runtime).await;

    let runtime_for_nav = shared_runtime.clone();
    let builder = WebviewBuilder::new(
        webview_label.to_string(),
        WebviewUrl::CustomProtocol(build_entry_src(&app, args.query.clone())),
    )
        .transparent(true)
        .on_navigation(move |url| {
            runtime_for_nav.with_runtime(|runtime| is_allowed_app_url(url, &runtime.app()))
        })
        .on_new_window(move |_url, _features| NewWindowResponse::Deny);

    let builder = build_storage(builder, &app)?;

    let (x, y, width, height) = if args.debug_layout {
        debug_layout_for_app(&app.id())
    } else {
        (0.0, 0.0, 1.0, 1.0)
    };
    let add_child_result = get_sage_window(app_handle)?
        .add_child(
            builder,
            LogicalPosition::new(x, y),
            LogicalSize::new(width, height),
        );
    if let Err(e) = add_child_result {
        let (runtime_id, app_id) = shared_runtime.with_runtime(|runtime| {
            (runtime.runtime_id(), runtime.app().id())
        });
        drop(shared_runtime);
        remove_runtime_by_runtime_id(apps_state, &runtime_id).await;
        remove_runtime_id_by_app_id(apps_state, &app_id).await;
        return Err(format!("failed to create child webview: {e}"));
    }

    if args.visibility == SageAppRuntimeVisibility::Hidden {
        let _ = get_webview_in_sage_window(app_handle, &webview_label)?.hide();
    }

    emit_runtime_manager_runtimes_changed(app_handle, apps_state).await;

    Ok(shared_runtime)
}

pub(in crate::runtime) async fn create_impostor_runtime_from_stopped(
    app_handle: AppHandle,
    apps_state: State<'_, AppsHostState>,
    stopped: &ResolvedStoppedApp,
    impostor_app: SharedSageApp,
    args: CreateImpostorRuntimeArgs,
) -> Result<SharedImpostorRuntime, String> {
    let victim_app = stopped.with_app(SharedSageApp::clone_for_runtime_owner);

    if !impostor_app.with(|app| app.common().is_sandbox_test())
        && !impostor_app.id().starts_with("__sage_runtime_")
    {
        return Err("impostor runtime app must be internal".into());
    }

    remove_impostor_runtime_by_victim_app_id(&apps_state, &victim_app.id()).await;

    let sage_window = get_sage_window(&app_handle)?;
    let webview_label = victim_app.webview_label();

    let runtime = SageAppRuntimeImpostorRecord::new(
        &victim_app,
        &impostor_app,
        sage_window.label(),
        &webview_label,
        args.kind,
    );

    let shared_runtime = write_impostor_runtime(&apps_state, runtime).await;

    let entry_url = build_entry_src_for(&victim_app, &impostor_app, args.query.clone());

    let builder = WebviewBuilder::new(
        webview_label.to_string(),
        WebviewUrl::CustomProtocol(entry_url.clone()),
    )
        .on_navigation(move |next_url| {
            *next_url == entry_url
        })
        .on_new_window(move |_url, _features| NewWindowResponse::Deny);

    let builder = build_persistent_storage_target(builder, &victim_app)?;

    let (x, y, width, height) = if args.debug_layout {
        debug_layout_for_app(&victim_app.id())
    } else {
        (0.0, 0.0, 1.0, 1.0)
    };

    let add_child_result = get_sage_window(&app_handle)?.add_child(
        builder,
        LogicalPosition::new(x, y),
        LogicalSize::new(width, height),
    );

    if let Err(err) = add_child_result {
        remove_impostor_runtime_by_victim_app_id(&apps_state, &victim_app.id()).await;
        return Err(format!("failed to create impostor child webview: {err}"));
    }

    if args.visibility == SageAppRuntimeVisibility::Hidden {
        let _ = get_webview_in_sage_window(&app_handle, &webview_label)?.hide();
    }

    Ok(shared_runtime)
}

fn persist_runtime_side_effects(app: &SharedSageApp) -> Result<(), String> {
    if app.is_system_app() {
        return Ok(());
    }

    write_installed_app_metadata(app)
        .map_err(|err| format!("failed to persist app runtime side effects: {err}"))
}

fn fallback_debug_slot(app_id: &str) -> usize {
    app_id.bytes().fold(0usize, |acc, b| {
        acc.wrapping_mul(31).wrapping_add(b as usize)
    }) % 12
}

fn debug_layout_for_app(app_id: &str) -> (f64, f64, f64, f64) {
    let slot = match app_id {
        "__sage_test_storage_isolation_persistent" => 0,
        "__sage_test_storage_isolation_incognito" => 1,
        "__sage_test_persistence_persistent" => 2,
        "__sage_test_persistence_incognito" => 3,
        "__sage_test_storage_clear_persistent" => 4,
        "__sage_test_network_allow_a" => 5,
        "__sage_test_network_allow_b" => 6,
        _ => fallback_debug_slot(app_id),
    };

    let cols = 3usize;
    let cell_w = 360.0;
    let cell_h = 100.0;
    let margin_x = 24.0;
    let margin_y = 24.0;
    let origin_x = 40.0;
    let origin_y = 40.0;

    let col = u32::try_from(slot % cols).expect("debug layout column should fit u32");
    let row = u32::try_from(slot / cols).expect("debug layout row should fit u32");

    let x = origin_x + f64::from(col) * (cell_w + margin_x);
    let y = origin_y + f64::from(row) * (cell_h + margin_y);

    (x, y, cell_w, cell_h)
}

async fn check_gates(apps_state: &State<'_, AppsHostState>, app: &SharedSageApp) -> Result<(), String> {
    let baseline = apps_state.sandbox.baseline.lock().await.clone();
    let current_run = apps_state.sandbox.current_run.lock().await.clone();
    let effective = sandbox::state_view::build_effective_state(&baseline, current_run.as_ref());
    let gate = sandbox::evaluate_app_launch_gate(app, &effective);

    if !gate.allowed {
        return Err(gate
            .message
            .unwrap_or_else(|| "App launch blocked by sandbox policy".into()));
    }

    Ok(())
}

fn build_storage(builder: WebviewBuilder<Wry>, app: &SharedSageApp) -> Result<WebviewBuilder<Wry>, String> {
    let has_persistent_storage = app.with(|app| {
        app.granted_permissions()
            .capabilities()
            .any(|cap| *cap == crate::capabilities::list::UserBridgeCapability::StoragePersistentWebview)
    });

    if !has_persistent_storage {
        return Ok(builder.incognito(true));
    }

    build_persistent_storage_target(builder, app)
}

fn build_persistent_storage_target(
    mut builder: WebviewBuilder<Wry>,
    app: &SharedSageApp,
) -> Result<WebviewBuilder<Wry>, String> {
    let storage = app.with(|app| app.storage().clone());

    match storage {
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        InstalledSageAppStorage::AppleDataStore { identifier_hex } => {
            let identifier = parse_data_store_id(&identifier_hex)?;
            builder = builder.data_store_identifier(identifier);
        }

        #[cfg(target_os = "windows")]
        InstalledSageAppStorage::WindowsProfile { directory_name } => {
            builder = builder.data_directory(crate::storage::data_directory_for(directory_name));
        }

        InstalledSageAppStorage::Unmanaged => {}

        #[allow(unreachable_patterns)]
        _ => {}
    }

    Ok(builder)
}

use std::collections::BTreeMap;

use specta::Type;
use tauri::webview::NewWindowResponse;
use tauri::{AppHandle, LogicalPosition, LogicalSize, State, WebviewBuilder, WebviewUrl, Wry};

#[cfg(target_os = "windows")]
use crate::data_directory_for;
#[cfg(any(target_os = "macos", target_os = "ios"))]
use crate::parse_data_store_id;
use crate::{
    AppMutationManager, AppPresentation, AppsHostState, CreateInstalledRuntimeArgs,
    OriginCleanupRuntimeTarget, ResolvedApp, RuntimeChangeSet, SageAppRuntimeMode,
    SageAppRuntimeRecord, SageAppRuntimeRecordView, SageAppRuntimeVisibility, SageAppStorage,
    SharedRuntime, SharedSageApp, build_entry_src, close_runtime_internal,
    emit_runtime_manager_runtimes_changed, focus_taskbar_runtime, get_sage_window,
    get_webview_in_sage_window, is_allowed_app_url, remove_runtime_by_runtime_id,
    remove_runtime_id_by_app_id, resolve_app, rotate_app_storage_and_origin, sandbox,
    sync_modal_runtime_visibility, write_runtime,
};

#[derive(Debug, Type)]
#[serde(rename_all = "camelCase")]
pub struct CreateRuntimeArgs {
    pub app_id: String,
    pub presentation: AppPresentation,
    pub mode: SageAppRuntimeMode,
    pub debug_layout: bool,
    pub query: BTreeMap<String, String>,
}

pub(crate) async fn start_user_app(
    app_handle: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    args: CreateInstalledRuntimeArgs,
) -> Result<SageAppRuntimeRecordView, String> {
    let created_runtime = create_runtime(
        app_handle,
        apps_state,
        CreateRuntimeArgs {
            app_id: args.app_id.clone(),
            presentation: AppPresentation::Taskbar,
            mode: SageAppRuntimeMode::Inline,
            debug_layout: false,
            query: BTreeMap::new(),
        },
    )
    .await
    .map(Into::into);

    emit_runtime_manager_runtimes_changed(app_handle, apps_state).await;

    if args.focus.unwrap_or(true) {
        focus_taskbar_runtime(app_handle, apps_state, &args.app_id).await?;
    }

    created_runtime
}

pub(crate) async fn start_system_app(
    app_handle: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    args: CreateRuntimeArgs,
) -> Result<SharedRuntime, String> {
    let runtime = create_runtime(app_handle, apps_state, args).await?;

    let host_window_label = runtime.with_runtime(SageAppRuntimeRecord::host_window_label);

    let mut changes = RuntimeChangeSet::default();
    changes.runtimes_changed();

    sync_modal_runtime_visibility(app_handle, apps_state, &host_window_label, &mut changes).await?;

    changes.emit(app_handle, apps_state).await;

    Ok(runtime)
}

pub(crate) async fn start_sandbox_test(
    app: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    app_id: &str,
    query: BTreeMap<String, String>,
) -> Result<(), String> {
    let args = CreateRuntimeArgs {
        app_id: app_id.to_string(),
        presentation: AppPresentation::Taskbar,
        mode: SageAppRuntimeMode::Inline,
        debug_layout: debug_test_apps_enabled(),
        query,
    };

    create_runtime(app, apps_state, args).await.map(|_| ())
}

pub(crate) async fn start_origin_cleanup_runtime(
    app_handle: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    target: OriginCleanupRuntimeTarget,
    query: BTreeMap<String, String>,
) -> Result<SharedRuntime, String> {
    let app_id = sandbox::BUILTIN_ORIGIN_CLEANUP_RUNTIME_ID;
    let mut app = sandbox::build_builtin_runtime_app(app_id)
        .map_err(|err| format!("failed to build origin cleanup runtime app: {err}"))?
        .ok_or_else(|| "missing origin cleanup runtime app".to_string())?;

    app.common_mut()
        .replace_storage_and_origin(target.storage, target.origin_id, false)
        .map_err(|err| format!("failed to retarget origin cleanup runtime: {err}"))?;

    close_runtime_internal(app_handle, apps_state, app_id).await;

    create_runtime_for_app(
        app_handle,
        apps_state,
        SharedSageApp::new(app),
        CreateRuntimeArgs {
            app_id: app_id.to_string(),
            presentation: AppPresentation::Taskbar,
            mode: SageAppRuntimeMode::Inline,
            debug_layout: false,
            query,
        },
        false,
        None,
    )
    .await
}

async fn create_runtime(
    app_handle: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    args: CreateRuntimeArgs,
) -> Result<SharedRuntime, String> {
    // Keep the per-app operation lock until the runtime and its webview are both
    // ready, so another start cannot observe the same app as stopped.
    let (app, operation_guard) = match resolve_app(app_handle, &args.app_id)
        .await
        .map_err(|e| e.to_string())?
    {
        ResolvedApp::Running(running) => return Ok(running.runtime()),
        ResolvedApp::Stopped(stopped) => stopped.into_app_and_guard(),
    };

    create_runtime_for_app(
        app_handle,
        apps_state,
        app,
        args,
        true,
        Some(operation_guard),
    )
    .await
}

async fn create_runtime_for_app(
    app_handle: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    app: SharedSageApp,
    args: CreateRuntimeArgs,
    apply_user_lifecycle: bool,
    operation_guard: Option<tokio::sync::OwnedMutexGuard<()>>,
) -> Result<SharedRuntime, String> {
    let is_internal = app.with(|app| app.common().is_sandbox_test())
        || app.id() == sandbox::BUILTIN_ORIGIN_CLEANUP_RUNTIME_ID;

    if !is_internal {
        check_gates(apps_state, &app).await?;
    }

    if apply_user_lifecycle {
        rotate_incognito_app_storage_and_origin_if_needed(app_handle, apps_state, &app).await?;
        mark_origin_may_contain_secrets_if_needed(app_handle, apps_state, &app).await?;
    }

    let sage_window = get_sage_window(app_handle)?;
    let webview_label = app.webview_label();
    let is_modal = matches!(args.presentation, AppPresentation::Modal(_));

    let runtime = SageAppRuntimeRecord::new(
        &app,
        sage_window.label(),
        &webview_label,
        args.presentation,
        args.mode,
        SageAppRuntimeVisibility::Hidden,
        is_internal,
    )
    .map_err(|err| err.to_string())?;

    let shared_runtime = write_runtime(apps_state, runtime).await;

    let runtime_for_nav = shared_runtime.clone();
    let builder = WebviewBuilder::new(
        webview_label.clone(),
        WebviewUrl::CustomProtocol(build_entry_src(&app, args.query.clone())),
    )
    .use_https_scheme(cfg!(target_os = "windows"))
    .transparent(!cfg!(target_os = "linux") || !is_modal)
    .on_navigation(move |url| {
        runtime_for_nav.with_runtime(|runtime| is_allowed_app_url(url, &runtime.app()))
    })
    .on_new_window(move |_url, _features| NewWindowResponse::Deny);

    let builder = build_initialization_script(builder);

    #[cfg(any(target_os = "macos", target_os = "ios"))]
    let builder = build_storage(builder, &app)?;

    #[cfg(not(any(target_os = "macos", target_os = "ios")))]
    let builder = build_storage(builder, &app);

    let (x, y, width, height) = if args.debug_layout {
        debug_layout_for_app(&app.id())
    } else {
        (0.0, 0.0, 1.0, 1.0)
    };

    let add_child_result = get_sage_window(app_handle)?.add_child(
        builder,
        LogicalPosition::new(x, y),
        LogicalSize::new(width, height),
    );

    if let Err(e) = add_child_result {
        let (runtime_id, app_id) =
            shared_runtime.with_runtime(|runtime| (runtime.runtime_id(), runtime.app().id()));
        drop(shared_runtime);
        remove_runtime_by_runtime_id(apps_state, &runtime_id).await;
        remove_runtime_id_by_app_id(apps_state, &app_id).await;
        return Err(format!("failed to create child webview: {e}"));
    }

    if !args.debug_layout {
        get_webview_in_sage_window(app_handle, &webview_label)?
            .hide()
            .map_err(|err| format!("{err}"))?;
    }

    drop(operation_guard);

    Ok(shared_runtime)
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

async fn check_gates(
    apps_state: &State<'_, AppsHostState>,
    app: &SharedSageApp,
) -> Result<(), String> {
    let baseline = apps_state.sandbox.baseline.lock().await.clone();
    let current_run = apps_state.sandbox.current_run.lock().await.clone();
    let effective = sandbox::build_effective_state(&baseline, current_run.as_ref());
    let gate = sandbox::evaluate_app_launch_gate(app, &effective);

    if !gate.allowed {
        tracing::error!("App launch blocked by sandbox policy");
        return Err(gate
            .message
            .unwrap_or_else(|| "App launch blocked by sandbox policy".into()));
    }

    Ok(())
}

async fn rotate_incognito_app_storage_and_origin_if_needed(
    app_handle: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    app: &SharedSageApp,
) -> Result<(), String> {
    if !app.is_user_app()
        || app.with(|app| app.common().is_sandbox_test())
        || app.with(|app| app.common().has_persistent_webview_storage())
    {
        return Ok(());
    }

    rotate_app_storage_and_origin(app_handle, apps_state, app).await
}

#[cfg(any(target_os = "macos", target_os = "ios"))]
fn build_storage(
    builder: WebviewBuilder<Wry>,
    app: &SharedSageApp,
) -> Result<WebviewBuilder<Wry>, String> {
    if !app.with(|app| app.common().has_persistent_webview_storage()) {
        return Ok(builder.incognito(true));
    }

    build_persistent_storage_target(builder, app)
}

#[cfg(target_os = "windows")]
fn build_storage(builder: WebviewBuilder<Wry>, app: &SharedSageApp) -> WebviewBuilder<Wry> {
    let builder = if app.with(|app| app.common().has_persistent_webview_storage()) {
        builder
    } else {
        builder.incognito(true)
    };

    // Incognito webviews still need their app-specific storage target. On Windows this scopes the
    // WebView2 InPrivate session so another incognito app cannot keep its in-memory data alive.
    build_windows_storage_target(builder, app)
}

#[cfg(not(any(target_os = "macos", target_os = "ios", target_os = "windows")))]
fn build_storage(builder: WebviewBuilder<Wry>, app: &SharedSageApp) -> WebviewBuilder<Wry> {
    if app.with(|app| app.common().has_persistent_webview_storage()) {
        builder
    } else {
        builder.incognito(true)
    }
}

#[cfg(any(target_os = "macos", target_os = "ios"))]
fn build_persistent_storage_target(
    builder: WebviewBuilder<Wry>,
    app: &SharedSageApp,
) -> Result<WebviewBuilder<Wry>, String> {
    let storage = app.with(|app| app.storage().clone());

    match storage {
        SageAppStorage::AppleDataStore { identifier_hex } => {
            let identifier = parse_data_store_id(&identifier_hex)?;
            Ok(builder.data_store_identifier(identifier))
        }

        SageAppStorage::WindowsProfile { .. } | SageAppStorage::Unmanaged => Ok(builder),
    }
}

#[cfg(target_os = "windows")]
fn build_windows_storage_target(
    builder: WebviewBuilder<Wry>,
    app: &SharedSageApp,
) -> WebviewBuilder<Wry> {
    let storage = app.with(|app| app.storage().clone());

    match storage {
        SageAppStorage::WindowsProfile { directory_name } => {
            builder.data_directory(data_directory_for(&directory_name))
        }

        SageAppStorage::AppleDataStore { .. } | SageAppStorage::Unmanaged => builder,
    }
}

fn build_initialization_script(mut builder: WebviewBuilder<Wry>) -> WebviewBuilder<Wry> {
    if !cfg!(debug_assertions) {
        return builder;
    }

    let enabled = std::env::var("SAGE_APPS_COMMS_DEBUG")
        .is_ok_and(|value| value == "1" || value.eq_ignore_ascii_case("true"));

    if !enabled {
        return builder;
    }

    builder = builder.initialization_script(
        r"
window.__SAGE_APPS_COMMS_DEBUG__ = true;
",
    );

    builder
}

async fn mark_origin_may_contain_secrets_if_needed(
    app_handle: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    app: &SharedSageApp,
) -> Result<(), String> {
    if app.is_system_app()
        || app.with(|app| app.common().is_sandbox_test())
        || !app.runtime_can_persist_secrets()
    {
        return Ok(());
    }

    if app.with(|app| app.common().origin_webview_storage_may_contain_secrets()) {
        return Ok(());
    }

    let manager = AppMutationManager::new(app_handle, apps_state);

    manager
        .mutate_shared_app(app, |ctx| {
            Box::pin(async move {
                ctx.draft_mut()
                    .mark_origin_webview_storage_may_contain_secrets()?;

                Ok(())
            })
        })
        .await
        .map_err(|err| format!("failed to mark origin as containing secrets: {err}"))
}

fn debug_test_apps_enabled() -> bool {
    cfg!(debug_assertions) && std::env::var("SAGE_DEBUG_TEST_APPS").is_ok_and(|v| v == "1")
}

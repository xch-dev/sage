use std::collections::BTreeMap;

use serde::Deserialize;
use specta::Type;
use tauri::webview::NewWindowResponse;
use tauri::{AppHandle, LogicalPosition, LogicalSize, State, WebviewUrl};

use crate::lifecycle::write_installed_app_metadata;
use crate::runtime::state::{
    SageAppRuntimeKind, SageAppRuntimeRecord, inline_label_for, runtime_id_for,
    write_runtime_and_emit_changed, write_runtime_id_by_app_id,
};
use crate::runtime::webview_locator::{
    find_webview_in_sage_window, get_sage_window, get_webview_in_sage_window,
};
use crate::runtime::{build_entry_src, is_allowed_app_url, resolve_app, runtime_kind_for_app};
use crate::storage::parse_data_store_id;
use crate::types::{InstalledSageAppStorage, SageApp};
use crate::{AppsHostState, sandbox};

#[derive(Debug, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CreateInlineRuntimeArgs {
    pub app_id: String,
    pub visible: bool,
    pub internal: bool,
    pub debug_layout: bool,
    pub path: Option<String>,
    pub query: BTreeMap<String, String>,
}

pub async fn create_inline_runtime(
    app: AppHandle,
    apps_state: State<'_, AppsHostState>,
    args: CreateInlineRuntimeArgs,
) -> Result<SageAppRuntimeRecord, String> {
    let mut resolved = resolve_app(&app, &args.app_id)?;
    let runtime_kind = runtime_kind_for_app(&resolved);
    let webview_label = inline_label_for(resolved.id(), runtime_kind);
    let runtime_id = runtime_id_for(resolved.id(), runtime_kind);
    let entry_src = build_entry_src(&resolved, args.path.clone(), args.query.clone());

    if let Some(existing) = find_webview_in_sage_window(&app, &webview_label) {
        return reuse_existing_inline_runtime(ReuseInlineRuntimeParams {
            apps_state: &apps_state,
            webview: &existing,
            runtime_id,
            app_id: resolved.id().to_string(),
            app_name: resolved.name().to_string(),
            entry_src,
            webview_label,
            runtime_kind,
            visible: args.visible,
            internal: args.internal,
        })
        .await;
    }

    if !args.internal && !resolved.is_sandbox_test() {
        let baseline = apps_state.sandbox.baseline.lock().await.clone();
        let current_run = apps_state.sandbox.current_run.lock().await.clone();
        let effective = sandbox::state_view::build_effective_state(&baseline, current_run.as_ref());
        let gate = sandbox::evaluate_app_launch_gate(&resolved, &effective);

        if !gate.allowed {
            return Err(gate
                .message
                .unwrap_or_else(|| "App launch blocked by sandbox policy".into()));
        }
    }

    let use_incognito = should_use_incognito(&resolved);
    let origin_id_for_nav = resolved.origin_id().to_string();
    let runtime_kind_for_nav = runtime_kind;

    let mut builder = tauri::webview::WebviewBuilder::new(
        &webview_label,
        WebviewUrl::CustomProtocol(
            entry_src
                .parse()
                .map_err(|e| format!("invalid entry url: {e}"))?,
        ),
    )
    .on_navigation(move |url| is_allowed_app_url(url, &origin_id_for_nav, runtime_kind_for_nav))
    .on_new_window(move |_url, _features| NewWindowResponse::Deny);

    if use_incognito {
        builder = builder.incognito(true);
    } else {
        match resolved.storage() {
            #[cfg(any(target_os = "macos", target_os = "ios"))]
            InstalledSageAppStorage::AppleDataStore { identifier_hex } => {
                let identifier = parse_data_store_id(identifier_hex)?;
                builder = builder.data_store_identifier(identifier);
            }

            #[cfg(target_os = "windows")]
            InstalledSageAppStorage::WindowsProfile { directory_name } => {
                builder =
                    builder.data_directory(crate::storage::data_directory_for(directory_name));
            }

            _ => {}
        }
    }

    let (x, y, width, height) = if args.debug_layout {
        debug_layout_for_app(resolved.id())
    } else {
        (0.0, 0.0, 1.0, 1.0)
    };

    get_sage_window(&app)?
        .add_child(
            builder,
            LogicalPosition::new(x, y),
            LogicalSize::new(width, height),
        )
        .map_err(|e| format!("failed to create child webview: {e}"))?;

    let record =
        SageAppRuntimeRecord::new_inline(&mut resolved, entry_src, args.visible, args.internal);

    persist_runtime_side_effects(&resolved)?;

    if !args.visible {
        let _ = get_webview_in_sage_window(&app, record.webview_label())?.hide();
    }

    write_runtime_id_by_app_id(&apps_state, &resolved, record.runtime_id().to_string()).await;
    write_runtime_and_emit_changed(&app, &apps_state, record.clone()).await;

    Ok(record)
}

fn persist_runtime_side_effects(app: &SageApp) -> Result<(), String> {
    let Some(user_app) = app.as_user() else {
        return Ok(());
    };

    write_installed_app_metadata(user_app)
        .map_err(|err| format!("failed to persist app runtime side effects: {err}"))
}

struct ReuseInlineRuntimeParams<'a> {
    apps_state: &'a State<'a, AppsHostState>,
    webview: &'a tauri::Webview,
    runtime_id: String,
    app_id: String,
    app_name: String,
    entry_src: String,
    webview_label: String,
    runtime_kind: SageAppRuntimeKind,
    visible: bool,
    internal: bool,
}

async fn reuse_existing_inline_runtime(
    params: ReuseInlineRuntimeParams<'_>,
) -> Result<SageAppRuntimeRecord, String> {
    let ReuseInlineRuntimeParams {
        apps_state,
        webview,
        runtime_id,
        app_id,
        app_name,
        entry_src,
        webview_label,
        runtime_kind,
        visible,
        internal,
    } = params;

    if visible {
        webview
            .show()
            .map_err(|e| format!("failed to show existing child webview: {e}"))?;
    } else {
        webview
            .hide()
            .map_err(|e| format!("failed to hide existing child webview: {e}"))?;
    }

    let mut record = {
        let by_runtime_id = apps_state.runtime.runtime_by_runtime_id.lock().await;
        by_runtime_id.get(&runtime_id).cloned()
    }
    .unwrap_or_else(|| {
        SageAppRuntimeRecord::new_existing_inline_fallback(
            runtime_id.clone(),
            app_id.clone(),
            app_name,
            entry_src,
            webview_label.clone(),
            runtime_kind,
            internal,
        )
    });

    record.mark_inline_reused(visible, internal);

    {
        let mut by_runtime_id = apps_state.runtime.runtime_by_runtime_id.lock().await;
        by_runtime_id.insert(runtime_id.clone(), record.clone());
    }

    {
        let mut runtime_by_app_id = apps_state.runtime.runtime_id_by_app_id.lock().await;
        runtime_by_app_id.insert(app_id, runtime_id);
    }

    Ok(record)
}

fn should_use_incognito(app: &SageApp) -> bool {
    let has_persistent_storage = app
        .granted_permissions()
        .capabilities()
        .any(|cap| *cap == crate::bridge::capabilities::UserBridgeCapability::PersistentStorage);

    !has_persistent_storage || app.flags().storage_may_contain_secrets()
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

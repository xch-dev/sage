use tauri::{AppHandle, Manager};

use crate::{
    BridgeOrigin, app_id_from_webview_label, get_webview_in_sage_window, is_allowed_app_url,
    protocol_scheme_for_app, resolve_running_app,
};

pub(crate) async fn assert_bridge_origin(
    app_handle: &AppHandle,
    webview_label: &String,
) -> Result<BridgeOrigin, String> {
    let app_id = app_id_from_webview_label(webview_label)
        .ok_or_else(|| format!("invalid app runtime label: {webview_label}"))?;

    let runtime = resolve_running_app(&app_handle.state(), app_id)
        .await
        .map_err(|_| format!("failed to find runtime for app {app_id}"))?;

    let app = runtime.into_app();

    if !app.webview_label_matches(webview_label) {
        return Err(format!(
            "bridge denied for {webview_label}: webview label mismatch"
        ));
    }

    let app_webview = get_webview_in_sage_window(app_handle, webview_label)?;

    let current_url = app_webview
        .url()
        .map_err(|e| format!("failed to read current webview url: {e}"))?;

    if !is_allowed_app_url(&current_url, &app) {
        return Err(format!(
            "bridge denied for {webview_label}: current url {} is outside {}://{}/...",
            current_url,
            protocol_scheme_for_app(&app),
            app.origin_id()
        ));
    }

    Ok(BridgeOrigin { app })
}

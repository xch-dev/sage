use tauri::{AppHandle, Manager};

const SAGE_WINDOW_LABEL: &str = "main";
const SAGE_WEBVIEW_LABEL: &str = "main";

pub(crate) fn find_sage_window(app: &AppHandle) -> Option<tauri::Window> {
    app.get_window(SAGE_WINDOW_LABEL)
}

pub(crate) fn get_sage_window(app: &AppHandle) -> Result<tauri::Window, String> {
    find_sage_window(app).ok_or_else(|| "missing sage window".to_string())
}

pub(crate) fn find_webview_in_sage_window(
    app: &AppHandle,
    webview_label: &str,
) -> Option<tauri::Webview> {
    find_sage_window(app)?.get_webview(webview_label)
}

pub(crate) fn get_webview_in_sage_window(
    app: &AppHandle,
    webview_label: &str,
) -> Result<tauri::Webview, String> {
    find_webview_in_sage_window(app, webview_label)
        .ok_or_else(|| format!("missing '{webview_label}' webview").to_string())
}

pub(crate) fn get_sage_webview(app: &AppHandle) -> Result<tauri::Webview, String> {
    get_webview_in_sage_window(app, SAGE_WEBVIEW_LABEL)
}

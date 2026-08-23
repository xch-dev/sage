use tauri::{AppHandle, Runtime, Webview, command};

use crate::Result;
use crate::SageExt;
use crate::models::*;

fn authorize_webview_control<R: Runtime>(caller: &Webview<R>, target_label: &str) -> Result<()> {
    if caller.label() != "main" {
        return Err(crate::Error::PermissionDenied(format!(
            "webview {} cannot control child webviews",
            caller.label()
        )));
    }

    if !is_sage_app_webview_label(target_label) {
        return Err(crate::Error::InvalidRequest(format!(
            "webview target is not a Sage app child: {target_label}"
        )));
    }

    Ok(())
}

#[command]
pub(crate) async fn is_ndef_available<R: Runtime>(
    app: AppHandle<R>,
) -> Result<IsNdefAvailableResponse> {
    app.sage().is_ndef_available()
}

#[command]
pub(crate) async fn get_ndef_payloads<R: Runtime>(
    app: AppHandle<R>,
) -> Result<GetNdefPayloadsResponse> {
    app.sage().get_ndef_payloads()
}

#[command]
pub(crate) async fn set_webview_bounds<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    request: SetWebviewBoundsRequest,
) -> Result<SetWebviewBoundsResponse> {
    authorize_webview_control(&webview, &request.label)?;

    #[cfg(target_os = "ios")]
    {
        return app.sage().set_webview_bounds(request);
    }

    #[cfg(not(target_os = "ios"))]
    {
        let _ = (app, request);
        Err(crate::Error::Unsupported(
            "native child webview bounds are only supported on iOS".to_string(),
        ))
    }
}

#[command]
pub(crate) async fn snapshot_webview<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    request: SnapshotWebviewRequest,
) -> Result<SnapshotWebviewResponse> {
    authorize_webview_control(&webview, &request.label)?;

    #[cfg(target_os = "ios")]
    {
        return app.sage().snapshot_webview(request);
    }

    #[cfg(not(target_os = "ios"))]
    {
        let _ = (app, request);
        Err(crate::Error::Unsupported(
            "native webview snapshots are only supported on iOS".to_string(),
        ))
    }
}

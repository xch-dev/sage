use std::{fs, path::PathBuf};

use anyhow::{Result as AnyResult, anyhow};
use tauri::http::{Request, Response, StatusCode};
use tauri::{AppHandle, Manager};

use crate::{
    app_id_from_webview_label,
    AppState,
    build_app_csp,
    resolve_running_app,
    ResolvedRunningApp,
    SharedSageApp,
};

pub async fn handle_user_app_protocol_request(
    app_handle: AppHandle,
    webview_label: String,
    request: Request<Vec<u8>>,
) -> Response<Vec<u8>> {
    let result = async {
        let runtime = get_protocol_request_runtime(&app_handle, &webview_label).await?;

        let app = runtime.into_app();
        if !app.is_user_app() {
            anyhow::bail!("not a user runtime");
        }

        let is_sandbox_test = app.with(|app| app.common().is_sandbox_test());

        match handle_app_protocol_request(&app_handle, &app, &request).await {
            Ok(response) => Ok(response),
            Err(err) if is_sandbox_test => Ok(protocol_error_response("sage-app", &err)),
            Err(_) => Ok(not_found_response()),
        }
    }
    .await;

    result.unwrap_or_else(|_| not_found_response())
}

pub async fn handle_system_app_protocol_request(
    app_handle: AppHandle,
    webview_label: String,
    request: Request<Vec<u8>>,
) -> Response<Vec<u8>> {
    let result = async {
        let runtime = get_protocol_request_runtime(&app_handle, &webview_label).await?;
        let app = runtime.into_app();
        if !app.is_system_app() {
            anyhow::bail!("not a system runtime");
        }

        handle_app_protocol_request(&app_handle, &app, &request)
            .await
            .map_err(|err| anyhow!("sage-system-app error: {err}"))
    }
    .await;

    result.unwrap_or_else(|err| protocol_error_response("sage-system-app", &err))
}

async fn get_protocol_request_runtime(
    app_handle: &AppHandle,
    webview_label: &str,
) -> AnyResult<ResolvedRunningApp> {
    let app_id =
        app_id_from_webview_label(webview_label).ok_or_else(|| anyhow!("invalid webview label"))?;

    resolve_running_app(&app_handle.state(), app_id)
        .await
        .map_err(|_| anyhow!("failed to find runtime for app {app_id}"))
}

async fn handle_app_protocol_request(
    app_handle: &AppHandle,
    app: &SharedSageApp,
    request: &Request<Vec<u8>>,
) -> AnyResult<Response<Vec<u8>>> {
    let file_path = protocol_file_path_for_request(app, request)?;

    let mime = mime_guess::from_path(&file_path)
        .first_or_octet_stream()
        .essence_str()
        .to_string();

    let network_id = active_network_id(app_handle).await?;

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", mime)
        .header("Cache-Control", "no-store")
        .header("Content-Security-Policy", build_app_csp(app, &network_id))
        .header("X-Content-Type-Options", "nosniff")
        .body(fs::read(&file_path)?)
        .map_err(|err| anyhow!("failed to build app protocol response: {err}"))
}

fn protocol_file_path_for_request(
    app: &SharedSageApp,
    request: &Request<Vec<u8>>,
) -> AnyResult<PathBuf> {
    if request.uri().host() != Some(&app.origin_id()) {
        anyhow::bail!("host mismatch");
    }

    if request.headers().contains_key("Service-Worker") {
        anyhow::bail!("Service worker forbidden");
    }

    let request_path = request.uri().path();

    app.with(|app| app.active_snapshot().resolve_file_path(request_path))
}

fn not_found_response() -> Response<Vec<u8>> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header("Content-Type", "text/plain; charset=utf-8")
        .body("Not found".to_string().into_bytes())
        .expect("failed to build error response")
}

fn protocol_error_response(prefix: &str, err: &anyhow::Error) -> Response<Vec<u8>> {
    tracing::error!(prefix, error = %err, "protocol request failed");

    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .header("Content-Type", "text/plain; charset=utf-8")
        .body(format!("{prefix} error: {err}").into_bytes())
        .expect("failed to build protocol error response")
}

async fn active_network_id(app_handle: &AppHandle) -> AnyResult<String> {
    use std::time::Duration;

    use tokio::time::timeout;

    const TIMEOUT: Duration = Duration::from_millis(500);

    let state = app_handle.state::<AppState>();

    let sage = timeout(TIMEOUT, state.lock())
        .await
        .map_err(|_| anyhow!("active network id unavailable because Sage state is locked"))?;

    Ok(sage.network_id())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use tempfile::{TempDir, tempdir};

    use super::*;
    use crate::{SageAppCommon, SageAppIdentity, SageAppManifestFile, SageAppManifestSageVersion, SageAppManifestVersion, SageAppPackageManifest, SageAppPackageManifestParts, SageAppSnapshot, SageAppStorage, SageAppUrl, SageAppWalletScope, SageGrantedPermissions, SageRequestedPermissions, UserSageApp, UserSageAppSource};

    fn manifest_file(path: &str) -> SageAppManifestFile {
        SageAppManifestFile::new(path, "a".repeat(64), 1).unwrap()
    }

    fn manifest() -> SageAppPackageManifest {
        SageAppPackageManifest::try_from(SageAppPackageManifestParts {
            manifest_version: SageAppManifestVersion(0),
            name: "test app".to_string(),
            icon: None,
            sage_version: SageAppManifestSageVersion {
                min: "0.0.0".to_string(),
                tested_max: None,
            },
            version: "1.0.0".to_string(),
            permissions: SageRequestedPermissions::empty(),
            files: vec![manifest_file("index.html"), manifest_file("nested/app.js")],
            entry: Some("index.html".to_string()),
            author: None,
            donation: None,
        })
        .unwrap()
    }

    fn app() -> (SharedSageApp, TempDir) {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("index.html"), "x").unwrap();
        fs::create_dir_all(dir.path().join("nested")).unwrap();
        fs::write(dir.path().join("nested/app.js"), "x").unwrap();

        let manifest = manifest();
        let granted =
            SageGrantedPermissions::new(manifest.permissions(), [], [], BTreeMap::default())
                .unwrap();
        let snapshot =
            SageAppSnapshot::new("hash", dir.path().to_string_lossy(), manifest).unwrap();
        let common = SageAppCommon::new(
            SageAppIdentity::new("app-id", "origin-id", dir.path().to_string_lossy()).unwrap(),
            granted,
            SageAppStorage::Unmanaged,
            snapshot,
            SageAppWalletScope::AllWallets,
        )
        .unwrap();
        let app = UserSageApp::new_installed(
            common,
            UserSageAppSource::Url {
                app_url: SageAppUrl::parse("https://example.com/app/").unwrap(),
            },
        );

        (SharedSageApp::new(app.into_sage_app()), dir)
    }

    fn request(uri: &str) -> Request<Vec<u8>> {
        Request::builder().uri(uri).body(Vec::new()).unwrap()
    }

    #[test]
    fn protocol_file_path_accepts_matching_host_and_snapshot_path() {
        let (app, _dir) = app();
        let request = request("sage-app://origin-id/nested/app.js");
        let expected = app.with(|app| app.active_snapshot().file_path("nested/app.js"));

        assert_eq!(
            protocol_file_path_for_request(&app, &request).unwrap(),
            expected.canonicalize().unwrap()
        );
    }

    #[test]
    fn protocol_file_path_rejects_host_mismatch() {
        let (app, _dir) = app();
        let request = request("sage-app://other-origin/index.html");

        let err = protocol_file_path_for_request(&app, &request).unwrap_err();

        assert!(
            err.to_string().contains("host mismatch"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn protocol_file_path_rejects_service_worker_requests() {
        let (app, _dir) = app();
        let request = Request::builder()
            .uri("sage-app://origin-id/index.html")
            .header("Service-Worker", "script")
            .body(Vec::new())
            .unwrap();

        let err = protocol_file_path_for_request(&app, &request).unwrap_err();

        assert!(
            err.to_string().contains("Service worker forbidden"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn protocol_file_path_rejects_traversal_paths() {
        let (app, _dir) = app();

        for uri in [
            "sage-app://origin-id/../secret.txt",
            "sage-app://origin-id/nested/../index.html",
        ] {
            assert!(
                protocol_file_path_for_request(&app, &request(uri)).is_err(),
                "expected {uri} to be rejected"
            );
        }
    }
}

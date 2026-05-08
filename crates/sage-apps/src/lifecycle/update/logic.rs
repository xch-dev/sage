use std::io;

use crate::host::Result;
use crate::lifecycle::{fetch_url_manifest, fetch_url_manifest_preview};
use crate::types::{ResolvedApp, SageApp, SageAppSnapshot, SageAppUrlPreview, SharedSageApp, UserSageAppPendingUpdate, UserSageAppSource};

pub async fn check_app_update_for_app(
    app: &ResolvedApp,
) -> Result<Option<SageAppUrlPreview>> {
    let shared_sage_app = app.clone_app_for_operation();
    let deps = shared_sage_app.with(|app| match app {
        SageApp::System(_) => None,
        SageApp::User(user_app) => Some((
            user_app.source().clone(),
            user_app.common().active_snapshot().clone(),
            user_app.pending_update().cloned(),
        )),
    });

    let Some((source, active_snapshot, existing_pending)) = deps else {
        return Ok(None);
    };

    let app_url = match source {
        UserSageAppSource::Url { app_url } => app_url,
        UserSageAppSource::Zip => return Ok(None),
    };

    let (manifest_preview, manifest_hash) = fetch_url_manifest_preview(&app_url.manifest_url())
        .await
        .map_err(|err| io::Error::other(format!("failed to fetch app manifest: {err}")))?;

    let preview = SageAppUrlPreview::new(&app_url, manifest_preview, manifest_hash)
        .await
        .map_err(|err| io::Error::other(format!("failed to preview app URL: {err}")))?;

    if preview.manifest_hash() == active_snapshot.manifest_hash()
        && let Some(full_manifest) = preview.full_manifest()
        && full_manifest == active_snapshot.manifest()
    {
        return Ok(None);
    }

    if let Some(existing_pending) = existing_pending
        && let Some(full_manifest) = preview.full_manifest()
        && existing_pending.manifest_hash() == preview.manifest_hash()
        && existing_pending.manifest() == full_manifest
    {
        return Ok(None);
    }

    Ok(Some(preview))
}

pub async fn fetch_pending_update(
    app: &SharedSageApp,
) -> Result<Option<UserSageAppPendingUpdate>> {
    struct FetchDeps {
        source: UserSageAppSource,
        active_snapshot: SageAppSnapshot,
    }

    let deps = app.with(|app| match app {
        SageApp::System(_) => None,
        SageApp::User(user_app) => Some(FetchDeps {
            source: user_app.source().clone(),
            active_snapshot: user_app.common().active_snapshot().clone(),
        }),
    });

    let Some(deps) = deps else {
        return Ok(None);
    };

    let app_url = match deps.source {
        UserSageAppSource::Url { app_url } => app_url.clone(),
        UserSageAppSource::Zip => return Ok(None),
    };

    let (manifest, manifest_hash) = fetch_url_manifest(&app_url.manifest_url())
        .await
        .map_err(|err| io::Error::other(format!("failed to fetch app manifest: {err}")))?;

    let preview = SageAppUrlPreview::from_full_manifest(&app_url, manifest, manifest_hash)
        .await
        .map_err(|err| io::Error::other(format!("failed to preview app URL: {err}")))?;

    let manifest = preview
        .require_full_manifest()
        .map_err(|err| io::Error::other(format!("update manifest is not installable: {err}")))?;

    if preview.manifest_hash() == deps.active_snapshot.manifest_hash()
        && manifest == deps.active_snapshot.manifest()
    {
        return Ok(None);
    }

    Ok(Some(UserSageAppPendingUpdate::new(
        app_url,
        preview.manifest_hash().to_string(),
        manifest.clone(),
    )))
}

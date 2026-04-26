use std::io;
use std::path::PathBuf;
use crate::host::Result;
use tauri::{command, AppHandle, State};
use crate::bridge::event_emit::emit_bridge_event_to_app_id;
use crate::bridge::methods::user::app::events::EventForApp;
use crate::bridge::USER_BRIDGE_CHANNEL;
use crate::host::AppState;
use crate::lifecycle::{download_url_snapshot, manifest_entry_file, manifest_icon_file, read_installed_app_by_id, write_installed_app_metadata};
use crate::lifecycle::install::url::preview_app_url_internal;
use crate::lifecycle::update::permissions::update_app_permissions_with_change_internal;
use crate::permissions::{resolve_capability_flags, resolve_effective_granted_capabilities};
use crate::permissions::capabilities::normalization::normalize_granted_capabilities;
use crate::permissions::capabilities::validation::validate_granted_capabilities;
use crate::permissions::network::normalize_and_validate_granted_network;
use crate::types::{SageAppUrlPreview, SageGrantedNetworkPermissions, SageGrantedPermissions, UserSageApp, UserSageAppPendingUpdate, UserSageAppSource};

#[command]
#[specta::specta]
pub async fn check_app_update(
    state: State<'_, AppState>,
    app_id: String,
) -> Result<Option<SageAppUrlPreview>> {
    let base_path = {
        let state = state.lock().await;
        state.path.clone()
    };

    let app = read_installed_app_by_id(&base_path, &app_id).map_err(|err| {
        io::Error::other(format!("failed to read installed app {}: {err}", app_id))
    })?;

    let app_url = match &app.source {
        UserSageAppSource::Url { app_url, .. } => app_url.clone(),
        UserSageAppSource::Zip => return Ok(None),
    };

    let preview = preview_app_url_internal(app_url)
        .await
        .map_err(|err| io::Error::other(format!("failed to preview app URL: {err}")))?;

    let same_manifest_hash =
        preview.manifest_hash == app.common.active_snapshot.manifest_hash;
    let same_manifest_content = preview.manifest == app.common.active_snapshot.manifest;

    if same_manifest_hash && same_manifest_content {
        return Ok(None);
    }

    if let Some(pending) = &app.pending_update {
        let same_pending_hash = pending.manifest_hash == preview.manifest_hash;
        let same_pending_manifest = pending.manifest == preview.manifest;

        if same_pending_hash && same_pending_manifest {
            return Ok(None);
        }
    }

    Ok(Some(preview))
}

#[command]
#[specta::specta]
pub async fn download_app_update(
    state: State<'_, AppState>,
    app_id: String,
) -> Result<UserSageApp> {
    let base_path = {
        let state = state.lock().await;
        state.path.clone()
    };

    let mut app = read_installed_app_by_id(&base_path, &app_id).map_err(|err| {
        io::Error::other(format!("failed to read installed app {}: {err}", app_id))
    })?;

    let (app_url, manifest_url) = match &app.source {
        UserSageAppSource::Url {
            app_url,
            manifest_url,
        } => (app_url.clone(), manifest_url.clone()),
        UserSageAppSource::Zip => {
            return Err(
                io::Error::other("zip apps do not support URL update download").into(),
            );
        }
    };

    let preview = match check_app_update(state, app_id.clone()).await? {
        Some(preview) => preview,
        None => return Ok(app),
    };

    app.pending_update = Some(UserSageAppPendingUpdate {
        app_url,
        manifest_url,
        manifest_hash: preview.manifest_hash,
        manifest: preview.manifest,
    });

    let app_dir = PathBuf::from(&app.common.app_dir);
    write_installed_app_metadata(&app, &app_dir)
        .map_err(|err| io::Error::other(format!("failed to write app metadata: {err}")))?;

    Ok(app)
}

#[command]
#[specta::specta]
pub async fn apply_app_update(
    state: State<'_, AppState>,
    app_id: String,
    granted_permissions: SageGrantedPermissions,
) -> Result<UserSageApp> {
    let base_path = {
        let state = state.lock().await;
        state.path.clone()
    };

    let mut app = read_installed_app_by_id(&base_path, &app_id).map_err(|err| {
        io::Error::other(format!("failed to read installed app {}: {err}", app_id))
    })?;

    let pending = app
        .pending_update
        .clone()
        .ok_or_else(|| io::Error::other(format!("app {} has no pending update", app_id)))?;

    let normalized_capabilities = normalize_granted_capabilities(
        &granted_permissions.capabilities,
    )
        .map_err(|err| io::Error::other(format!("invalid granted permissions for update: {err}")))?;

    validate_granted_capabilities(
        &pending.manifest.permissions,
        &normalized_capabilities,
    )
        .map_err(|err| io::Error::other(format!("invalid granted permissions for update: {err}")))?;

    let granted_network_whitelist = normalize_and_validate_granted_network(
        &pending.manifest.permissions.network,
        &granted_permissions.network.whitelist,
    )
        .map_err(|err| {
            io::Error::other(format!("invalid granted network whitelist for update: {err}"))
        })?;

    let effective_capabilities = resolve_effective_granted_capabilities(
        &pending.manifest.permissions,
        &normalized_capabilities,
    )
        .map_err(|err| {
            io::Error::other(format!("invalid granted permission policy for update: {err}"))
        })?;

    let permission_flags = resolve_capability_flags(
        &effective_capabilities,
        Some(&app.common.capability_flags),
    )
        .map_err(|err| {
            io::Error::other(format!("invalid granted permission policy for update: {err}"))
        })?;

    let app_dir = PathBuf::from(&app.common.app_dir);

    let snapshot = download_url_snapshot(
        &app_dir,
        &pending.app_url,
        &pending.manifest,
        &pending.manifest_hash,
    )
        .await
        .map_err(|err| io::Error::other(format!("failed to download update snapshot: {err}")))?;

    app.common.name = pending.manifest.name.clone();
    app.common.version = pending.manifest.version.clone();
    app.common.requested_permissions = pending.manifest.permissions.clone();

    app.common.granted_permissions = SageGrantedPermissions {
        capabilities: normalized_capabilities,
        network: SageGrantedNetworkPermissions {
            whitelist: granted_network_whitelist,
        },
    };

    app.common.capability_flags = permission_flags;
    app.common.active_snapshot = snapshot;
    app.common.entry_file =
        manifest_entry_file(&app.common.active_snapshot.manifest).to_string();
    app.common.icon_file =
        manifest_icon_file(&app.common.active_snapshot.manifest).to_string();
    app.pending_update = None;

    write_installed_app_metadata(&app, &app_dir)
        .map_err(|err| io::Error::other(format!("failed to write app metadata: {err}")))?;

    Ok(app)
}

#[command]
#[specta::specta]
pub async fn apps_update_permissions(
    app: AppHandle,
    state: State<'_, AppState>,
    app_id: String,
    granted_permissions: SageGrantedPermissions,
    clear_storage_taint: bool,
) -> Result<()> {
    let base_path = {
        let state = state.lock().await;
        state.path.clone()
    };

    let (_updated, capability_change, network_change) =
        update_app_permissions_with_change_internal(
            &base_path,
            &app_id,
            granted_permissions,
            clear_storage_taint,
        )
            .map_err(|err| io::Error::other(format!("failed to update app permissions: {err}")))?;

    if !capability_change.added.is_empty() || !capability_change.removed.is_empty() {
        let _ = emit_bridge_event_to_app_id(
            &app,
            &app_id,
            EventForApp::from_capabilities_change(USER_BRIDGE_CHANNEL, capability_change)
        ).await;
    }

    if !network_change.added.is_empty() || !network_change.removed.is_empty() {
        let _ = emit_bridge_event_to_app_id(
            &app,
            &app_id,
            EventForApp::from_network_whitelist_change(USER_BRIDGE_CHANNEL, network_change)
        ).await;
    }

    Ok(())
}

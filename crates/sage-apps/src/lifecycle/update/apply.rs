use std::collections::BTreeMap;
use std::{fs, io};

use tauri::{AppHandle, Manager, State};

use crate::{
    AppMutationManager, AppsHostState, CreateInstalledRuntimeArgs, ResolvedApp, Result, SageApp,
    SageAppView, SageGrantedPermissionsInput, SharedSageApp, UserSageAppPendingUpdate,
    download_url_snapshot, emit_listed_apps_changed, emit_pending_update_changed,
    find_active_taskbar_runtime, find_runtime_by_app_id_optional, fresh_snapshot_dir,
    get_sage_window, resolve_app, resolve_stopped_app, start_app_update_runtime, start_user_app,
    write_snapshot_manifest,
};

pub(crate) async fn apply_app_update_inner(
    app_handle: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    app_id: &str,
    additional_granted_permissions_input: Option<SageGrantedPermissionsInput>,
    expected_manifest_hash: Option<&str>,
) -> Result<SageAppView> {
    let _update_guard = apps_state
        .try_begin_app_update(app_id)
        .map_err(io::Error::other)?;

    let preflight = preflight_apply_app_update(app_handle, apps_state, app_id).await?;

    if preflight.should_review && additional_granted_permissions_input.is_none() {
        open_update_runtime(
            app_handle,
            apps_state,
            app_id,
            None,
            "failed to start app update review runtime",
        )
        .await;

        let resolved = resolve_app(app_handle, app_id).await.map_err(|err| {
            io::Error::other(format!("failed to read installed app {app_id}: {err}"))
        })?;

        return Ok(resolved.with_app(|app| app.into()));
    }

    ensure_pending_update_matches_review(
        app_id,
        preflight.pending.manifest_hash(),
        expected_manifest_hash,
    )?;

    let reopen_after_update = ReopenAfterUpdate::capture(app_handle, apps_state, app_id).await?;

    let app = execute_app_update(
        app_handle,
        app_id,
        preflight.pending,
        additional_granted_permissions_input,
    )
    .await?;

    emit_pending_update_changed(app_handle, apps_state, &app).await;
    emit_listed_apps_changed(app_handle, apps_state).await;

    if reopen_after_update.should_reopen
        && let Err(err) = start_user_app(
            app_handle,
            apps_state,
            CreateInstalledRuntimeArgs {
                app_id: app_id.to_string(),
                focus: Some(reopen_after_update.should_focus),
            },
        )
        .await
    {
        tracing::error!(
            error = %err,
            app_id = %app_id,
            focus = reopen_after_update.should_focus,
            "failed to reopen app runtime after update"
        );
    }

    Ok(app.into())
}

pub(crate) async fn try_auto_apply_pending_update(
    app_handle: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    app_id: &str,
) -> Result<bool> {
    let should_attempt_apply = {
        let resolved = resolve_app(app_handle, app_id).await.map_err(|err| {
            io::Error::other(format!("failed to read installed app {app_id}: {err}"))
        })?;

        if let ResolvedApp::Running(_) = resolved {
            return Ok(false);
        }

        let has_pending_update = resolved.with_app(|app| {
            app.with(|sage_app| {
                sage_app
                    .as_user()
                    .and_then(|user_app| user_app.pending_update())
                    .is_some()
            })
        });

        if !has_pending_update {
            return Ok(false);
        }

        let should_review = resolved.with_app(SharedSageApp::should_review_pending_update);

        !should_review
    };

    if !should_attempt_apply {
        return Ok(false);
    }

    apply_app_update_inner(app_handle, apps_state, app_id, None, None).await?;

    Ok(true)
}

fn ensure_pending_update_matches_review(
    app_id: &str,
    pending_manifest_hash: &str,
    expected_manifest_hash: Option<&str>,
) -> Result<()> {
    if let Some(expected) = expected_manifest_hash
        && pending_manifest_hash != expected
    {
        return Err(io::Error::other(format!(
            "pending update for {app_id} changed since it was reviewed; re-check the update before applying"
        ))
        .into());
    }

    Ok(())
}

async fn execute_app_update(
    app_handle: &AppHandle,
    app_id: &str,
    pending: UserSageAppPendingUpdate,
    additional_granted_permissions_input: Option<SageGrantedPermissionsInput>,
) -> Result<SharedSageApp> {
    let apps_state: State<'_, AppsHostState> = app_handle.state();

    let resolved = resolve_stopped_app(app_handle, app_id)
        .await
        .map_err(|err| {
            io::Error::other(format!(
                "failed to resolve stopped app {app_id} for update: {err}"
            ))
        })?;

    let app = resolved.into_app();

    let granted_permissions_input = app
        .try_with(|sage_app| {
            let base = SageGrantedPermissionsInput::from((
                sage_app.common().granted_permissions(),
                pending.manifest().permissions(),
            ));

            Ok::<_, anyhow::Error>(match additional_granted_permissions_input {
                Some(additional) => base.with_additional(additional),
                None => base,
            })
        })
        .map_err(io::Error::other)?;

    let app_dir = app.with(SageApp::app_path);

    let old_snapshot_dir =
        app.with(|app| app.common().active_snapshot().snapshot_dir().to_string());

    let snapshot_dir = fresh_snapshot_dir(&app_dir);

    fs::create_dir_all(&snapshot_dir).map_err(|err| {
        io::Error::other(format!(
            "failed to create update snapshot directory {}: {err}",
            snapshot_dir.display()
        ))
    })?;

    let snapshot = download_url_snapshot(
        &snapshot_dir,
        pending.app_url(),
        pending.manifest(),
        pending.manifest_hash(),
    )
    .await
    .map_err(|err| io::Error::other(format!("failed to download update snapshot: {err}")))?;

    write_snapshot_manifest(&snapshot).map_err(|err| {
        io::Error::other(format!("failed to write update snapshot manifest: {err}"))
    })?;

    let new_snapshot_dir = snapshot.snapshot_dir().to_string();

    let granted_permissions = granted_permissions_input
        .resolve(pending.manifest().permissions())
        .map_err(|err| io::Error::other(format!("invalid update permissions: {err}")))?;

    let manager = AppMutationManager::new(app_handle, &apps_state);

    manager
        .mutate_shared_app(&app, move |ctx| {
            Box::pin(async move {
                ctx.draft_mut()
                    .app_mut()
                    .apply_update(&pending, granted_permissions, snapshot)?;

                ctx.draft_mut().app_mut().set_pending_update(None)?;

                Ok(())
            })
        })
        .await
        .map_err(io::Error::other)?;

    if old_snapshot_dir != new_snapshot_dir {
        let _ = fs::remove_dir_all(old_snapshot_dir);
    }

    Ok(app)
}

async fn open_update_runtime(
    app_handle: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    app_id: &str,
    issue: Option<String>,
    error_message: &'static str,
) {
    let mut query = BTreeMap::new();
    query.insert("appId".to_string(), app_id.to_string());

    if let Some(issue) = issue {
        query.insert("issue".to_string(), issue);
    }

    let _ = start_app_update_runtime(app_handle, apps_state, app_id.to_string(), query)
        .await
        .map_err(|err| {
            tracing::error!(
                error = %err,
                app_id = %app_id,
                "{error_message}"
            );
            err
        });
}

struct ApplyAppUpdatePreflight {
    pending: UserSageAppPendingUpdate,
    should_review: bool,
}

async fn preflight_apply_app_update(
    app_handle: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    app_id: &str,
) -> Result<ApplyAppUpdatePreflight> {
    let resolved = resolve_app(app_handle, app_id)
        .await
        .map_err(|err| io::Error::other(format!("failed to read installed app {app_id}: {err}")))?;

    let pending = resolved.with_app(|app| {
        app.try_with(|sage_app| {
            let user_app = sage_app
                .as_user()
                .ok_or_else(|| anyhow::anyhow!("system app cannot receive user update"))?;

            user_app
                .pending_update()
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("app {app_id} has no pending update"))
        })
    });

    let pending = match pending {
        Ok(pending) => pending,
        Err(err) => {
            let app = resolved.clone_app_for_operation();
            emit_pending_update_changed(app_handle, apps_state, &app).await;

            return Err(io::Error::other(err).into());
        }
    };

    let should_review = resolved.with_app(SharedSageApp::should_review_pending_update);

    Ok(ApplyAppUpdatePreflight {
        pending,
        should_review,
    })
}

#[derive(Debug, Clone, Copy)]
struct ReopenAfterUpdate {
    should_reopen: bool,
    should_focus: bool,
}

impl ReopenAfterUpdate {
    async fn capture(
        app_handle: &AppHandle,
        apps_state: &State<'_, AppsHostState>,
        app_id: &str,
    ) -> anyhow::Result<Self> {
        let should_reopen = find_runtime_by_app_id_optional(apps_state, app_id)
            .await
            .is_some();

        if !should_reopen {
            return Ok(Self {
                should_reopen: false,
                should_focus: false,
            });
        }

        let sage_window = get_sage_window(app_handle).map_err(|err| anyhow::anyhow!(err))?;
        let active_taskbar_runtime =
            find_active_taskbar_runtime(apps_state, sage_window.label()).await;
        let should_focus = active_taskbar_runtime.is_some_and(|runtime| runtime.app_id() == app_id);

        Ok(Self {
            should_reopen: true,
            should_focus,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::ensure_pending_update_matches_review;

    #[test]
    fn reviewed_manifest_hash_must_match_pending_update() {
        ensure_pending_update_matches_review("app.test", "current", None).unwrap();
        ensure_pending_update_matches_review("app.test", "current", Some("current")).unwrap();

        let err = ensure_pending_update_matches_review("app.test", "current", Some("reviewed"))
            .expect_err("a stale review must not authorize a different pending update");

        assert!(
            err.to_string().contains("changed since it was reviewed"),
            "unexpected error: {err}"
        );
    }
}

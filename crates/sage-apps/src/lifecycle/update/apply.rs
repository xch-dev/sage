use std::collections::BTreeMap;
use std::path::PathBuf;
use std::{fs, io};

use tauri::{AppHandle, Manager, State};

use crate::{
    AppMutationManager, AppsHostState, CreateInstalledRuntimeArgs, ResolvedApp, Result, SageApp,
    SageAppCompatibility, SageAppCompatibilityStatus, SageAppView, SageGrantedPermissionsInput,
    SharedSageApp, UserSageAppPendingUpdate, UserSageAppPendingUpdateDecisionView,
    check_app_update_inner, download_url_snapshot, emit_listed_apps_changed,
    emit_pending_update_changed, fetch_recoverable_app_update, find_active_taskbar_runtime,
    find_runtime_by_app_id_optional, fresh_snapshot_dir, get_sage_window, resolve_app,
    resolve_stopped_app, start_app_update_runtime, start_user_app, write_snapshot_manifest,
};

pub(crate) enum RecoverableAppUpdateOutcome {
    Applied(Box<SageAppView>),
    ReviewOpened,
    NotReady,
}

pub(crate) async fn apply_recoverable_app_update_inner(
    app_handle: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    app_id: &str,
    additional_granted_permissions_input: Option<SageGrantedPermissionsInput>,
    expected_manifest_hash: Option<&str>,
    open_review_when_needed: bool,
) -> Result<RecoverableAppUpdateOutcome> {
    let _update_guard = apps_state
        .try_begin_app_update(app_id)
        .map_err(io::Error::other)?;

    let operation_lock = apps_state.operation_lock_for_app(app_id);
    let _operation_guard = operation_lock.lock().await;

    let Some((recoverable, preview)) = fetch_recoverable_app_update(apps_state, app_id).await?
    else {
        return Ok(RecoverableAppUpdateOutcome::NotReady);
    };

    let Some(manifest) = preview.full_manifest() else {
        if open_review_when_needed {
            open_update_runtime(
                app_handle,
                apps_state,
                app_id,
                None,
                "failed to open recovery update details",
            )
            .await;
            return Ok(RecoverableAppUpdateOutcome::ReviewOpened);
        }

        return Ok(RecoverableAppUpdateOutcome::NotReady);
    };

    let crate::UserSageAppSource::Url { app_url } = recoverable.source() else {
        return Ok(RecoverableAppUpdateOutcome::NotReady);
    };
    let pending = UserSageAppPendingUpdate::new(
        app_url.clone(),
        preview.manifest_hash().to_string(),
        manifest.clone(),
    );

    ensure_pending_update_matches_review(app_id, pending.manifest_hash(), expected_manifest_hash)?;

    let compatibility =
        SageAppCompatibility::for_app(app_handle, pending.manifest().sage_version());
    let decision = UserSageAppPendingUpdateDecisionView::from_pending_update(
        recoverable.granted_permissions(),
        pending.manifest().permissions(),
    );
    let needs_review = recovery_update_needs_review(&decision, &compatibility);

    if needs_review && additional_granted_permissions_input.is_none() {
        if open_review_when_needed {
            open_update_runtime(
                app_handle,
                apps_state,
                app_id,
                None,
                "failed to open recovery update review",
            )
            .await;

            return Ok(RecoverableAppUpdateOutcome::ReviewOpened);
        }

        return Ok(RecoverableAppUpdateOutcome::NotReady);
    }

    compatibility
        .ensure_installable()
        .map_err(io::Error::other)?;

    let granted_permissions = resolve_updated_permissions(
        recoverable.granted_permissions(),
        pending.manifest().permissions(),
        additional_granted_permissions_input,
    )
    .map_err(|err| io::Error::other(format!("invalid recovery update permissions: {err}")))?;

    let app_dir = PathBuf::from(recoverable.app_dir());
    let old_snapshot_dir = recoverable.active_snapshot_dir().to_string();
    let snapshot_dir = fresh_snapshot_dir(&app_dir);
    let mut snapshot_dir_cleanup = SnapshotDirCleanup(Some(snapshot_dir.clone()));

    let snapshot = download_url_snapshot(
        &snapshot_dir,
        pending.app_url(),
        pending.manifest(),
        pending.manifest_hash(),
    )
    .await
    .map_err(|err| io::Error::other(format!("failed to download recovery snapshot: {err}")))?;

    write_snapshot_manifest(&snapshot)
        .map_err(|err| io::Error::other(format!("failed to write recovery manifest: {err}")))?;

    let new_snapshot_dir = snapshot.snapshot_dir().to_string();
    let repaired = recoverable
        .repair(granted_permissions, snapshot)
        .map_err(|err| io::Error::other(format!("failed to rebuild app after update: {err}")))?;

    let mut tx = apps_state.db.begin_immediate().await.map_err(|err| {
        io::Error::other(format!(
            "failed to begin app recovery transaction for {app_id}: {err}"
        ))
    })?;

    if let Err(err) = tx.persist_user_app(&repaired).await {
        tx.rollback().await;
        return Err(
            io::Error::other(format!("failed to persist repaired app {app_id}: {err}")).into(),
        );
    }

    let reread = match tx.load_user_app(app_id).await {
        Ok(app) => app,
        Err(err) => {
            tx.rollback().await;
            return Err(
                io::Error::other(format!("failed to verify repaired app {app_id}: {err}")).into(),
            );
        }
    };

    tx.commit().await.map_err(|err| {
        io::Error::other(format!("failed to commit repaired app {app_id}: {err}"))
    })?;

    snapshot_dir_cleanup.preserve();

    if old_snapshot_dir != new_snapshot_dir {
        let _ = fs::remove_dir_all(old_snapshot_dir);
    }

    let shared = SharedSageApp::new(SageApp::User(reread));
    emit_pending_update_changed(app_handle, apps_state, &shared).await;
    emit_listed_apps_changed(app_handle, apps_state).await;

    Ok(RecoverableAppUpdateOutcome::Applied(Box::new(
        shared.into(),
    )))
}

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

    if preflight.compatibility.ensure_installable().is_err() {
        open_update_runtime(
            app_handle,
            apps_state,
            app_id,
            None,
            "failed to open incompatible app update details",
        )
        .await;

        let resolved = resolve_app(app_handle, app_id).await.map_err(|err| {
            io::Error::other(format!("failed to read installed app {app_id}: {err}"))
        })?;

        return Ok(resolved.with_app(|app| app.into()));
    }

    let compatibility_needs_review = matches!(
        preflight.compatibility.status(),
        SageAppCompatibilityStatus::UntestedNewerSage { .. }
    );

    if (preflight.should_review || compatibility_needs_review)
        && additional_granted_permissions_input.is_none()
    {
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

    let mut reopen_after_update = ReopenAfterUpdate::default();

    let update_result = execute_app_update(
        app_handle,
        app_id,
        preflight.pending,
        additional_granted_permissions_input,
        &mut reopen_after_update,
    )
    .await;

    if reopen_after_update.should_reopen
        && let Err(reopen_err) = start_user_app(
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
            error = %reopen_err,
            app_id = %app_id,
            focus = reopen_after_update.should_focus,
            "failed to reopen app runtime after update"
        );
    }

    let app = update_result?;
    emit_pending_update_changed(app_handle, apps_state, &app).await;
    emit_listed_apps_changed(app_handle, apps_state).await;

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

        let pending_compatibility = resolved.with_app(|app| {
            app.with(|sage_app| {
                sage_app
                    .as_user()
                    .and_then(|user_app| user_app.pending_update())
                    .map(|pending| {
                        SageAppCompatibility::for_app(app_handle, pending.manifest().sage_version())
                    })
            })
        });

        let Some(pending_compatibility) = pending_compatibility else {
            return Ok(false);
        };

        if pending_compatibility.ensure_installable().is_err()
            || matches!(
                pending_compatibility.status(),
                SageAppCompatibilityStatus::UntestedNewerSage { .. }
            )
        {
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
    reopen_after_update: &mut ReopenAfterUpdate,
) -> Result<SharedSageApp> {
    SageAppCompatibility::for_app(app_handle, pending.manifest().sage_version())
        .ensure_installable()
        .map_err(io::Error::other)?;

    let apps_state: State<'_, AppsHostState> = app_handle.state();

    let resolved = resolve_app(app_handle, app_id)
        .await
        .map_err(|err| io::Error::other(format!("failed to resolve app {app_id}: {err}")))?;
    let app = resolved.clone_app_for_operation();
    drop(resolved);

    let (granted_permissions_input, expected_granted_permissions) = app
        .try_with(|sage_app| {
            let current_granted_permissions = sage_app.common().granted_permissions();
            let base = SageGrantedPermissionsInput::from((
                current_granted_permissions,
                pending.manifest().permissions(),
            ));

            let input = match additional_granted_permissions_input {
                Some(additional) => base.with_additional(additional),
                None => base,
            };

            Ok::<_, anyhow::Error>((input, current_granted_permissions.clone()))
        })
        .map_err(io::Error::other)?;

    let app_dir = app.with(SageApp::app_path);

    let old_snapshot_dir =
        app.with(|app| app.common().active_snapshot().snapshot_dir().to_string());

    let snapshot_dir = fresh_snapshot_dir(&app_dir);
    let mut snapshot_dir_cleanup = SnapshotDirCleanup(Some(snapshot_dir.clone()));

    let snapshot = download_url_snapshot(
        &snapshot_dir,
        pending.app_url(),
        pending.manifest(),
        pending.manifest_hash(),
    )
    .await;

    if snapshot.is_err()
        && let Err(err) = check_app_update_inner(app_handle, &apps_state, app_id).await
    {
        tracing::warn!(error = %err, app_id = %app_id, "failed to refresh app update");
    }

    let snapshot = snapshot.map_err(|err| {
        io::Error::other(format!(
            "The update files could not be downloaded. Sage left the installed app unchanged: {err}"
        ))
    })?;

    write_snapshot_manifest(&snapshot).map_err(|err| {
        io::Error::other(format!("failed to write update snapshot manifest: {err}"))
    })?;

    let new_snapshot_dir = snapshot.snapshot_dir().to_string();

    *reopen_after_update = ReopenAfterUpdate::capture(app_handle, &apps_state, app_id).await?;

    let resolved = resolve_stopped_app(app_handle, app_id)
        .await
        .map_err(|err| {
            io::Error::other(format!(
                "failed to stop app {app_id} after preparing its update: {err}"
            ))
        })?;

    let (app, operation_guard) = resolved.into_app_and_guard();

    let granted_permissions = granted_permissions_input
        .resolve(pending.manifest().permissions())
        .map_err(|err| io::Error::other(format!("invalid update permissions: {err}")))?;

    let pending_manifest_hash = pending.manifest_hash().to_string();
    let manager = AppMutationManager::new(app_handle, &apps_state);

    manager
        .mutate_shared_app(&app, move |ctx| {
            Box::pin(async move {
                let current_manifest_hash = ctx
                    .draft()
                    .app()
                    .as_user()
                    .and_then(|app| app.pending_update())
                    .map(UserSageAppPendingUpdate::manifest_hash);

                if current_manifest_hash != Some(pending_manifest_hash.as_str()) {
                    anyhow::bail!("a newer update became available; try updating again");
                }

                if ctx.draft().app().common().granted_permissions()
                    != &expected_granted_permissions
                {
                    anyhow::bail!(
                        "app permissions changed while the update was being prepared; try updating again"
                    );
                }

                ctx.draft_mut()
                    .app_mut()
                    .apply_update(&pending, granted_permissions, snapshot)?;

                ctx.draft_mut().app_mut().set_pending_update(None)?;

                Ok(())
            })
        })
        .await
        .map_err(io::Error::other)?;

    snapshot_dir_cleanup.preserve();

    if old_snapshot_dir != new_snapshot_dir {
        let _ = fs::remove_dir_all(old_snapshot_dir);
    }

    drop(operation_guard);

    Ok(app)
}

struct SnapshotDirCleanup(Option<PathBuf>);

impl SnapshotDirCleanup {
    fn preserve(&mut self) {
        self.0 = None;
    }
}

impl Drop for SnapshotDirCleanup {
    fn drop(&mut self) {
        if let Some(path) = self.0.take() {
            let _ = fs::remove_dir_all(path);
        }
    }
}

fn resolve_updated_permissions(
    active_grants: &crate::SageGrantedPermissions,
    requested: &crate::SageRequestedPermissions,
    additional: Option<SageGrantedPermissionsInput>,
) -> anyhow::Result<crate::SageGrantedPermissions> {
    let retained = SageGrantedPermissionsInput::from((active_grants, requested));
    let granted = match additional {
        Some(additional) => retained.with_additional(additional),
        None => retained,
    };

    granted.resolve(requested)
}

fn recovery_update_needs_review(
    decision: &UserSageAppPendingUpdateDecisionView,
    compatibility: &SageAppCompatibility,
) -> bool {
    decision.is_review()
        || matches!(
            compatibility.status(),
            SageAppCompatibilityStatus::UntestedNewerSage { .. }
                | SageAppCompatibilityStatus::Invalid { .. }
                | SageAppCompatibilityStatus::RequiresNewerSage { .. }
        )
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
    compatibility: SageAppCompatibility,
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
    let compatibility =
        SageAppCompatibility::for_app(app_handle, pending.manifest().sage_version());

    Ok(ApplyAppUpdatePreflight {
        pending,
        should_review,
        compatibility,
    })
}

#[derive(Debug, Clone, Copy, Default)]
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
    use std::collections::BTreeMap;

    use semver::Version;
    use tempfile::tempdir;

    use super::{
        SnapshotDirCleanup, ensure_pending_update_matches_review, recovery_update_needs_review,
        resolve_updated_permissions,
    };
    use crate::{
        SageAppCompatibility, SageAppManifestSageVersion, SageGrantedPermissions,
        SageGrantedPermissionsInput, SageRequestedCapabilities, SageRequestedNetworkPermissions,
        SageRequestedPermissions, UserBridgeCapability, UserSageAppPendingUpdateDecisionView,
    };

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

    #[test]
    fn snapshot_directory_is_removed_unless_preserved() {
        let root = tempdir().unwrap();

        for (name, preserve) in [("removed", false), ("preserved", true)] {
            let path = root.path().join(name);
            std::fs::create_dir_all(&path).unwrap();

            let mut cleanup = SnapshotDirCleanup(Some(path.clone()));
            if preserve {
                cleanup.preserve();
            }
            drop(cleanup);

            assert_eq!(path.exists(), preserve);
        }
    }

    #[test]
    fn updated_permissions_retain_only_still_requested_grants_and_add_reviewed_grants() {
        let old_requested = SageRequestedPermissions::new(
            SageRequestedNetworkPermissions::empty(),
            SageRequestedCapabilities::new([], [UserBridgeCapability::StoragePersistentWebview]),
        )
        .unwrap();
        let active_grants = SageGrantedPermissions::new(
            &old_requested,
            [UserBridgeCapability::StoragePersistentWebview],
            [],
            BTreeMap::new(),
        )
        .unwrap();

        let pending_requested = SageRequestedPermissions::new(
            SageRequestedNetworkPermissions::empty(),
            SageRequestedCapabilities::new([], [UserBridgeCapability::WalletSendXch]),
        )
        .unwrap();
        let additional = SageGrantedPermissionsInput::new(
            [UserBridgeCapability::WalletSendXch],
            [],
            BTreeMap::new(),
        );

        let resolved =
            resolve_updated_permissions(&active_grants, &pending_requested, Some(additional))
                .unwrap();

        assert!(resolved.has_capability(UserBridgeCapability::WalletSendXch));
        assert!(!resolved.has_capability(UserBridgeCapability::StoragePersistentWebview));
    }

    #[test]
    fn automatic_recovery_requires_compatible_update_without_permission_review() {
        let decision = UserSageAppPendingUpdateDecisionView::Apply;
        let compatible = SageAppCompatibility::evaluate(
            &Version::parse("0.13.0").unwrap(),
            &SageAppManifestSageVersion {
                min: "0.12.0".to_string(),
                tested_max: Some("0.13.0".to_string()),
            },
        );
        let untested = SageAppCompatibility::evaluate(
            &Version::parse("0.13.1").unwrap(),
            &SageAppManifestSageVersion {
                min: "0.12.0".to_string(),
                tested_max: Some("0.13.0".to_string()),
            },
        );

        assert!(!recovery_update_needs_review(&decision, &compatible));
        assert!(recovery_update_needs_review(&decision, &untested));

        let review = UserSageAppPendingUpdateDecisionView::from_pending_update(
            &SageGrantedPermissions::default(),
            &SageRequestedPermissions::new(
                SageRequestedNetworkPermissions::empty(),
                SageRequestedCapabilities::new(
                    [UserBridgeCapability::StoragePersistentWebview],
                    [],
                ),
            )
            .unwrap(),
        );

        assert!(recovery_update_needs_review(&review, &compatible));
    }
}

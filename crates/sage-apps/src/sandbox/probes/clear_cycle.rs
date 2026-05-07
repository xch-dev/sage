use crate::AppsHostState;
use crate::runtime::{resolve_stopped_app, run_verified_storage_clear_cycle};
use crate::runtime::stop::close_runtime_internal;
use crate::sandbox::BUILTIN_STORAGE_CLEAR_PERSISTENT_ID;
use crate::sandbox::probes::poll::{poll_persistence_read, poll_persistence_write};
use crate::sandbox::runtime::{start_test_app, unique_run_id};
use tauri::{AppHandle, State};

pub(in crate::sandbox) async fn run_clear_cycle_test(
    app: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
) -> Result<(bool, Option<String>), String> {
    let app_id = BUILTIN_STORAGE_CLEAR_PERSISTENT_ID;
    let run_id = unique_run_id("storage-clear-cycle-victim");

    start_test_app(
        app,
        apps_state,
        app_id,
        &[("runId", run_id.clone()), ("phase", "write".into())],
    )
        .await?;

    let write_results = poll_persistence_write(apps_state, &run_id, 1, 2_000).await?;
    let Some(write) = write_results.into_iter().find(|item| item.app_id == app_id) else {
        let () = close_runtime_internal(app, apps_state, app_id).await;
        return Ok((false, Some("Timed out waiting for victim storage write result.".into())));
    };

    let () = close_runtime_internal(app, apps_state, app_id).await;

    if write.data.error.is_some() || !write.data.local_storage_wrote || !write.data.indexed_db_wrote {
        return Ok((
            false,
            Some(write.data.error.unwrap_or_else(|| {
                "Victim storage write probe failed.".into()
            })),
        ));
    }

    let resolved_app = resolve_stopped_app(app, app_id)
        .await
        .map_err(|err| err.to_string())?;

    run_verified_storage_clear_cycle(app, &resolved_app).await?;

    drop(resolved_app);

    start_test_app(
        app,
        apps_state,
        app_id,
        &[("runId", run_id.clone()), ("phase", "read".into())],
    )
        .await?;

    let read_results = poll_persistence_read(apps_state, &run_id, 1, 2_000).await?;
    let Some(read) = read_results.into_iter().find(|item| item.app_id == app_id) else {
        let () = close_runtime_internal(app, apps_state, app_id).await;
        return Ok((false, Some("Timed out waiting for victim storage read result.".into())));
    };

    let () = close_runtime_internal(app, apps_state, app_id).await;

    if read.data.error.is_some() {
        return Ok((false, read.data.error));
    }

    let passed = !read.data.local_storage_present && !read.data.indexed_db_present;

    Ok((
        passed,
        Some(if passed {
            "Storage clear cycle removed data written and read back through the victim app.".into()
        } else {
            "Storage clear cycle completed, but victim app still saw localStorage or IndexedDB data afterwards.".into()
        }),
    ))
}

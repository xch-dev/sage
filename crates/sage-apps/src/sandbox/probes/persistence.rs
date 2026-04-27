use tauri::{AppHandle, State};

use crate::sandbox::{BUILTIN_PERSISTENCE_INCOGNITO_ID, BUILTIN_PERSISTENCE_PERSISTENT_ID};
use crate::state::AppsHostState;

use super::super::runtime::{start_test_app, stop_test_apps, unique_run_id};
use super::poll::{poll_persistence_read, poll_persistence_write};

pub async fn run_persistence_test(
    app: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
) -> Result<((bool, Option<String>), (bool, Option<String>)), String> {
    let run_id = unique_run_id("sandbox-persistence");
    let app_ids = [
        BUILTIN_PERSISTENCE_PERSISTENT_ID,
        BUILTIN_PERSISTENCE_INCOGNITO_ID,
    ];

    stop_test_apps(app, apps_state, &app_ids).await;

    start_test_app(
        app,
        apps_state,
        BUILTIN_PERSISTENCE_PERSISTENT_ID,
        &[("runId", run_id.clone()), ("phase", "write".into())],
        None,
    )
    .await?;
    start_test_app(
        app,
        apps_state,
        BUILTIN_PERSISTENCE_INCOGNITO_ID,
        &[("runId", run_id.clone()), ("phase", "write".into())],
        None,
    )
    .await?;

    let write_results = poll_persistence_write(apps_state, &run_id, 2, 10_000).await?;
    stop_test_apps(app, apps_state, &app_ids).await;

    let persistent_write = write_results
        .iter()
        .find(|r| r.app_id == BUILTIN_PERSISTENCE_PERSISTENT_ID);
    let incognito_write = write_results
        .iter()
        .find(|r| r.app_id == BUILTIN_PERSISTENCE_INCOGNITO_ID);

    let Some(persistent_write) = persistent_write else {
        return Ok((
            (false, Some("Missing persistent write result.".into())),
            (false, Some("Missing incognito write result.".into())),
        ));
    };
    let Some(incognito_write) = incognito_write else {
        return Ok((
            (false, Some("Missing persistent write result.".into())),
            (false, Some("Missing incognito write result.".into())),
        ));
    };

    if persistent_write.data.error.is_some()
        || !persistent_write.data.local_storage_wrote
        || !persistent_write.data.indexed_db_wrote
    {
        return Ok((
            (
                false,
                Some(
                    persistent_write
                        .data
                        .error
                        .clone()
                        .unwrap_or_else(|| "Persistent write probe failed.".into()),
                ),
            ),
            (
                false,
                Some(
                    persistent_write
                        .data
                        .error
                        .clone()
                        .unwrap_or_else(|| "Persistent write probe failed.".into()),
                ),
            ),
        ));
    }

    if incognito_write.data.error.is_some()
        || !incognito_write.data.local_storage_wrote
        || !incognito_write.data.indexed_db_wrote
    {
        return Ok((
            (
                false,
                Some(
                    incognito_write
                        .data
                        .error
                        .clone()
                        .unwrap_or_else(|| "Incognito write probe failed.".into()),
                ),
            ),
            (
                false,
                Some(
                    incognito_write
                        .data
                        .error
                        .clone()
                        .unwrap_or_else(|| "Incognito write probe failed.".into()),
                ),
            ),
        ));
    }

    start_test_app(
        app,
        apps_state,
        BUILTIN_PERSISTENCE_PERSISTENT_ID,
        &[("runId", run_id.clone()), ("phase", "read".into())],
        None,
    )
    .await?;
    start_test_app(
        app,
        apps_state,
        BUILTIN_PERSISTENCE_INCOGNITO_ID,
        &[("runId", run_id.clone()), ("phase", "read".into())],
        None,
    )
    .await?;

    let read_results = poll_persistence_read(apps_state, &run_id, 2, 10_000).await?;
    stop_test_apps(app, apps_state, &app_ids).await;

    let persistent_read = read_results
        .iter()
        .find(|r| r.app_id == BUILTIN_PERSISTENCE_PERSISTENT_ID);
    let incognito_read = read_results
        .iter()
        .find(|r| r.app_id == BUILTIN_PERSISTENCE_INCOGNITO_ID);

    let Some(persistent_read) = persistent_read else {
        return Ok((
            (false, Some("Missing persistent read result.".into())),
            (false, Some("Missing incognito read result.".into())),
        ));
    };
    let Some(incognito_read) = incognito_read else {
        return Ok((
            (false, Some("Missing persistent read result.".into())),
            (false, Some("Missing incognito read result.".into())),
        ));
    };

    let persistent_ok = persistent_read.data.error.is_none()
        && persistent_read.data.local_storage_present
        && persistent_read.data.indexed_db_present;

    let incognito_ok = incognito_read.data.error.is_none()
        && !incognito_read.data.local_storage_present
        && !incognito_read.data.indexed_db_present;

    Ok((
        (
            persistent_ok,
            Some(if persistent_ok {
                "Persistent mode retained localStorage and IndexedDB across reopen.".into()
            } else {
                persistent_read
                    .data
                    .error
                    .clone()
                    .unwrap_or_else(|| "Persistent read probe mismatch.".into())
            }),
        ),
        (
            incognito_ok,
            Some(if incognito_ok {
                "Incognito mode did not retain localStorage or IndexedDB across reopen.".into()
            } else {
                incognito_read
                    .data
                    .error
                    .clone()
                    .unwrap_or_else(|| "Incognito read probe mismatch.".into())
            }),
        ),
    ))
}

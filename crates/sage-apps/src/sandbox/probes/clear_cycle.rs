use crate::AppsHostState;
use crate::sandbox::BUILTIN_STORAGE_CLEAR_PERSISTENT_ID;
use tauri::{AppHandle, State};
use crate::runtime::{resolve_stopped_app, run_verified_storage_clear_cycle};

pub(in crate::sandbox) async fn run_clear_cycle_test(
    app: &AppHandle,
    _apps_state: &State<'_, AppsHostState>,
) -> Result<(bool, Option<String>), String> {
    let resolved_app = resolve_stopped_app(app, BUILTIN_STORAGE_CLEAR_PERSISTENT_ID)
        .await
        .map_err(|err| err.to_string())?;

    match run_verified_storage_clear_cycle(app, &resolved_app).await {
        Ok(()) => Ok((
            true,
            Some("Storage clear cycle removed localStorage and IndexedDB for the target app origin.".into()),
        )),
        Err(err) => Ok((false, Some(err))),
    }
}

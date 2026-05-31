use tauri::{AppHandle, State};

use crate::{AppsHostState, emit_user_runtime_event_to_listeners, EnvironmentThemeChangedEvent, EnvironmentThemeView};

#[tauri::command]
#[specta::specta]
pub async fn apps_set_environment_theme(
    app_handle: AppHandle,
    apps_state: State<'_, AppsHostState>,
    theme: EnvironmentThemeView,
) -> Result<(), String> {
    {
        let mut current = apps_state.environment.theme.current.lock().await;
        *current = Some(theme.clone());
    }

    emit_user_runtime_event_to_listeners(
        &app_handle,
        &apps_state,
        EnvironmentThemeChangedEvent {
            theme: theme.clone(),
        },
    )
    .await;

    Ok(())
}

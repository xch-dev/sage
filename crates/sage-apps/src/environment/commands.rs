use tauri::{AppHandle, State};

use crate::AppsHostState;
use crate::bridge::methods::user::environment::EnvironmentThemeView;
use crate::environment::theme::apply_environment_theme;

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

    apply_environment_theme(&app_handle, &apps_state, theme).await
}

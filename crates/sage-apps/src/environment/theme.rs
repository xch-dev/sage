use tauri::{AppHandle, State};

use crate::AppsHostState;
use crate::bridge::capabilities::UserBridgeCapability;
use crate::bridge::event_emit::emit_runtime_event_to_app_id;
use crate::bridge::methods::user::environment::{EnvironmentThemeChangedEvent, EnvironmentThemeView};
use crate::runtime::list_runtimes;

pub async fn apply_environment_theme(
    app_handle: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    theme: EnvironmentThemeView,
) -> Result<(), String> {
    let runtimes = list_runtimes(apps_state).await?;

    for runtime in runtimes {
        let (app_id, should_emit_event) = runtime.with_runtime(|record| {
            let app = record.app();

            (
                app.id(),
                app.is_capability_granted(UserBridgeCapability::EnvironmentThemeListenChanged),
            )
        });

        if should_emit_event {
            let _ = emit_runtime_event_to_app_id(
                app_handle,
                &app_id,
                EnvironmentThemeChangedEvent {
                    theme: theme.clone(),
                },
            )
                .await;
        }
    }

    Ok(())
}

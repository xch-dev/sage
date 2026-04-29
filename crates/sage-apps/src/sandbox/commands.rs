use tauri::{AppHandle, State};

use crate::AppsHostState;
use crate::runtime::resolve_app;

use super::gate::evaluate_app_launch_gate;
use super::runner::{begin_sandbox_run, sandbox_runner};
use super::state_view::{build_effective_state, build_state_view};
use super::types::{AppLaunchGateResult, SandboxStateView};

#[tauri::command]
#[specta::specta]
pub async fn apps_get_sandbox_state(
    apps_state: State<'_, AppsHostState>,
) -> Result<SandboxStateView, String> {
    let baseline = apps_state.sandbox.baseline.lock().await.clone();
    let current_run = apps_state.sandbox.current_run.lock().await.clone();

    Ok(build_state_view(&baseline, current_run.as_ref()))
}

#[tauri::command]
#[specta::specta]
pub async fn apps_get_app_launch_gate(
    app_handle: AppHandle,
    apps_state: State<'_, AppsHostState>,
    app_id: String,
) -> Result<AppLaunchGateResult, String> {
    let app = resolve_app(&app_handle, &app_id)?;

    let baseline = apps_state.sandbox.baseline.lock().await.clone();
    let current_run = apps_state.sandbox.current_run.lock().await.clone();

    let effective = build_effective_state(&baseline, current_run.as_ref());

    let evaluated_gate = evaluate_app_launch_gate(&app, &effective);
    
    Ok(evaluated_gate)
}

#[tauri::command]
#[specta::specta]
pub async fn apps_rerun_sandbox_tests(
    app: AppHandle,
    apps_state: State<'_, AppsHostState>,
) -> Result<SandboxStateView, String> {
    let view = begin_sandbox_run(&app, &apps_state).await?;

    let runner_app = app.clone();
    tokio::spawn(async move {
        let runner = Box::pin(sandbox_runner(runner_app));
        runner.await;
    });

    Ok(view)
}

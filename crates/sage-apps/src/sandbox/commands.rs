use tauri::{AppHandle, State, command};

use crate::{
    AppLaunchGateResult, AppsHostState, SandboxStateView, begin_sandbox_run, build_effective_state,
    build_state_view, evaluate_app_launch_gate, resolve_app, sandbox_runner,
};

#[command]
#[specta::specta]
pub async fn apps_get_sandbox_state(
    apps_state: State<'_, AppsHostState>,
) -> Result<SandboxStateView, String> {
    Ok(build_state_view(&apps_state).await)
}

#[command]
#[specta::specta]
pub async fn apps_get_app_launch_gate(
    app_handle: AppHandle,
    apps_state: State<'_, AppsHostState>,
    app_id: String,
) -> Result<AppLaunchGateResult, String> {
    let resolved_app = resolve_app(&app_handle, &app_id)
        .await
        .map_err(|_| "app not found".to_string())?;

    let baseline = apps_state.sandbox.baseline.lock().await.clone();
    let current_run = apps_state.sandbox.current_run.lock().await.clone();

    let effective = build_effective_state(&baseline, current_run.as_ref());

    let evaluated_gate = resolved_app.with_app(|app| evaluate_app_launch_gate(app, &effective));

    Ok(evaluated_gate)
}

#[command]
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

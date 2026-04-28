use serde_json::Value;
use tauri::State;

use crate::AppsHostState;

use super::store::{SandboxAppResult, replace_by_app_id};
use super::types::{
    SandboxIsolationProbeResult, SandboxNetworkProbeResult, SandboxPersistenceReadProbeResult,
    SandboxPersistenceWriteProbeResult, SandboxStorageClearProbeResult,
};

pub async fn ingest_bridge_send_payload(
    app_id: &str,
    payload: &Value,
    apps_state: &State<'_, AppsHostState>,
) {
    let Some(kind) = payload.get("kind").and_then(Value::as_str) else {
        return;
    };

    if kind != "sandbox_report" {
        return;
    }

    let Some(report) = payload.get("report") else {
        return;
    };

    let Some(report_type) = report.get("type").and_then(Value::as_str) else {
        return;
    };

    let Some(data) = report.get("data").cloned() else {
        return;
    };

    let run_id = data
        .get("runId")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();

    if run_id.is_empty() {
        return;
    }

    let mut runs = apps_state.sandbox.runs.lock().await;
    let run = runs.entry(run_id).or_default();

    match report_type {
        "isolation" => {
            if let Ok(parsed) = serde_json::from_value::<SandboxIsolationProbeResult>(data) {
                replace_by_app_id(
                    &mut run.isolation,
                    SandboxAppResult {
                        app_id: app_id.to_string(),
                        data: parsed,
                    },
                );
            }
        }
        "persistence_write" => {
            if let Ok(parsed) = serde_json::from_value::<SandboxPersistenceWriteProbeResult>(data) {
                replace_by_app_id(
                    &mut run.persistence_write,
                    SandboxAppResult {
                        app_id: app_id.to_string(),
                        data: parsed,
                    },
                );
            }
        }
        "persistence_read" => {
            if let Ok(parsed) = serde_json::from_value::<SandboxPersistenceReadProbeResult>(data) {
                replace_by_app_id(
                    &mut run.persistence_read,
                    SandboxAppResult {
                        app_id: app_id.to_string(),
                        data: parsed,
                    },
                );
            }
        }
        "network" => {
            if let Ok(parsed) = serde_json::from_value::<SandboxNetworkProbeResult>(data) {
                replace_by_app_id(
                    &mut run.network,
                    SandboxAppResult {
                        app_id: app_id.to_string(),
                        data: parsed,
                    },
                );
            }
        }
        "clear_cycle" => {
            if let Ok(parsed) = serde_json::from_value::<SandboxStorageClearProbeResult>(data) {
                replace_by_app_id(
                    &mut run.clear_cycle,
                    SandboxAppResult {
                        app_id: app_id.to_string(),
                        data: parsed,
                    },
                );
            }
        }
        _ => {}
    }
}

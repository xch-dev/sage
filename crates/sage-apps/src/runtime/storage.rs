use std::collections::BTreeMap;
use std::sync::OnceLock;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};
use tokio::sync::{Mutex, oneshot};
use tokio::time::timeout;

use crate::{AppsHostState, BUILTIN_ORIGIN_CLEANUP_RUNTIME_ID, close_runtime_internal, SageAppStorage, start_origin_cleanup_runtime};

#[derive(Debug, Clone)]
pub struct OriginCleanupRuntimeTarget {
    pub origin_id: String,
    pub storage: SageAppStorage,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OriginCleanupBridgePayload {
    pub kind: String,
    pub cleanup_id: String,
    pub ok: bool,
    #[serde(default)]
    pub errors: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct OriginCleanupResult {
    pub ok: bool,
    pub errors: Vec<String>,
}

type PendingOriginCleanups = Mutex<BTreeMap<String, oneshot::Sender<OriginCleanupResult>>>;

fn pending_origin_cleanups() -> &'static PendingOriginCleanups {
    static PENDING: OnceLock<PendingOriginCleanups> = OnceLock::new();
    PENDING.get_or_init(Default::default)
}

async fn register_pending_origin_cleanup(
    cleanup_id: String,
) -> oneshot::Receiver<OriginCleanupResult> {
    let (tx, rx) = oneshot::channel();

    pending_origin_cleanups()
        .lock()
        .await
        .insert(cleanup_id, tx);

    rx
}

async fn remove_pending_origin_cleanup(cleanup_id: &str) {
    pending_origin_cleanups().lock().await.remove(cleanup_id);
}

async fn wait_for_origin_cleanup_result(
    cleanup_id: &str,
    rx: oneshot::Receiver<OriginCleanupResult>,
    timeout_ms: u64,
) -> Result<OriginCleanupResult, String> {
    match timeout(Duration::from_millis(timeout_ms), rx).await {
        Ok(Ok(result)) => Ok(result),
        Ok(Err(_)) => Err(format!(
            "origin cleanup channel closed before result for {cleanup_id}"
        )),
        Err(_) => Err(format!(
            "timed out waiting for origin cleanup result for {cleanup_id}"
        )),
    }
}

pub(crate) async fn run_origin_cleanup(
    app: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
    target: OriginCleanupRuntimeTarget,
) -> anyhow::Result<()> {
    let cleanup_id = uuid::Uuid::new_v4().to_string();
    let rx = register_pending_origin_cleanup(cleanup_id.clone()).await;

    let mut query = BTreeMap::new();
    query.insert("cleanupId".to_string(), cleanup_id.clone());

    let start_result = start_origin_cleanup_runtime(app, apps_state, target, query).await;

    if let Err(err) = start_result {
        remove_pending_origin_cleanup(&cleanup_id).await;
        anyhow::bail!("failed to start origin cleanup runtime: {err}");
    }

    let result = wait_for_origin_cleanup_result(&cleanup_id, rx, 10_000)
        .await
        .map_err(anyhow::Error::msg);

    close_runtime_internal(app, apps_state, BUILTIN_ORIGIN_CLEANUP_RUNTIME_ID).await;

    let result = result?;

    if !result.ok {
        anyhow::bail!("origin cleanup failed: {:?}", result.errors);
    }

    Ok(())
}

pub(crate) async fn ingest_origin_cleanup_bridge_send_payload(
    app_id: &str,
    payload: &serde_json::Value,
    _host_state: &AppsHostState,
) -> anyhow::Result<()> {
    if app_id != BUILTIN_ORIGIN_CLEANUP_RUNTIME_ID {
        anyhow::bail!("origin cleanup payload came from unexpected app {app_id}");
    }

    let payload: OriginCleanupBridgePayload = serde_json::from_value(payload.clone())?;

    if payload.kind != "originCleanup.completed" {
        return Ok(());
    }

    let Some(tx) = pending_origin_cleanups()
        .lock()
        .await
        .remove(&payload.cleanup_id)
    else {
        tracing::warn!(
            app_id = %app_id,
            cleanup_id = %payload.cleanup_id,
            "origin cleanup result received without pending waiter"
        );
        return Ok(());
    };

    let _ = tx.send(OriginCleanupResult {
        ok: payload.ok,
        errors: payload.errors,
    });

    Ok(())
}

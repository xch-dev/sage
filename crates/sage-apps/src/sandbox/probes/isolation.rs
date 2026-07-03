use std::time::Duration;

use tauri::{AppHandle, State};

use super::super::runtime::{start_test_app, stop_test_apps, unique_run_id};
use super::poll::poll_isolation;
use crate::{
    AppsHostState, BUILTIN_STORAGE_ISOLATION_INCOGNITO_ID, BUILTIN_STORAGE_ISOLATION_PERSISTENT_ID,
    get_sage_webview,
};

/// Writes probe data into the host (main Sage) webview's `localStorage` and
/// `indexedDB` before the isolation probes run. Without this, the probe apps
/// read fixed keys that nothing ever wrote, so the test passed vacuously even
/// if storage isolation were completely broken. With the host seeded, a sandbox
/// app that can observe these values proves an isolation failure.
async fn seed_host_isolation_probe(app: &AppHandle, run_id: &str) -> Result<(), String> {
    let webview = get_sage_webview(app)?;

    let value = serde_json::to_string(run_id)
        .map_err(|err| format!("failed to encode isolation probe value: {err}"))?;

    let script = format!(
        r#"(function () {{
  var value = {value};
  try {{ localStorage.setItem('sage_probe_local_storage', value); }} catch (e) {{}}
  try {{
    var open = indexedDB.open('sage_probe_db', 1);
    open.onupgradeneeded = function () {{
      try {{
        var db = open.result;
        if (!db.objectStoreNames.contains('probe_store')) {{
          db.createObjectStore('probe_store');
        }}
      }} catch (e) {{}}
    }};
    open.onsuccess = function () {{
      try {{
        var db = open.result;
        if (!db.objectStoreNames.contains('probe_store')) {{ db.close(); return; }}
        var tx = db.transaction('probe_store', 'readwrite');
        tx.objectStore('probe_store').put(value, 'sage_probe_key');
        tx.oncomplete = function () {{ db.close(); }};
      }} catch (e) {{}}
    }};
  }} catch (e) {{}}
}})();"#
    );

    webview
        .eval(script)
        .map_err(|err| format!("failed to seed host isolation probe data: {err}"))
}

pub(in crate::sandbox) async fn run_isolation_test(
    app: &AppHandle,
    apps_state: &State<'_, AppsHostState>,
) -> Result<(bool, Option<String>), String> {
    let run_id = unique_run_id("sandbox-isolation");
    let app_ids = [
        BUILTIN_STORAGE_ISOLATION_PERSISTENT_ID,
        BUILTIN_STORAGE_ISOLATION_INCOGNITO_ID,
    ];

    stop_test_apps(app, apps_state, &app_ids).await;

    // Seed the host storage first, then give the async IndexedDB write a brief
    // moment to commit before launching the sandbox probes.
    seed_host_isolation_probe(app, &run_id).await?;
    tokio::time::sleep(Duration::from_millis(250)).await;

    start_test_app(
        app,
        apps_state,
        BUILTIN_STORAGE_ISOLATION_PERSISTENT_ID,
        &[("runId", run_id.clone())],
    )
    .await?;
    start_test_app(
        app,
        apps_state,
        BUILTIN_STORAGE_ISOLATION_INCOGNITO_ID,
        &[("runId", run_id.clone())],
    )
    .await?;

    let results = poll_isolation(apps_state, &run_id, 2, 2_000).await?;
    stop_test_apps(app, apps_state, &app_ids).await;

    let persistent = results
        .iter()
        .find(|r| r.app_id == BUILTIN_STORAGE_ISOLATION_PERSISTENT_ID);
    let incognito = results
        .iter()
        .find(|r| r.app_id == BUILTIN_STORAGE_ISOLATION_INCOGNITO_ID);

    let Some(persistent) = persistent else {
        return Ok((false, Some("Missing persistent isolation result.".into())));
    };
    let Some(incognito) = incognito else {
        return Ok((false, Some("Missing incognito isolation result.".into())));
    };

    for (label, result) in [
        ("persistent", &persistent.data),
        ("incognito", &incognito.data),
    ] {
        if result.error.is_some() {
            return Ok((
                false,
                Some(format!(
                    "{label} isolation probe reported error: {}",
                    result.error.clone().unwrap_or_default()
                )),
            ));
        }

        if result.local_storage_visible || result.indexed_db_visible {
            return Ok((
                false,
                Some(format!(
                    "{label} probe was able to observe Sage probe data."
                )),
            ));
        }
    }

    Ok((
        true,
        Some(
            "Both sandbox probe modes were unable to observe the seeded Sage host probe data."
                .into(),
        ),
    ))
}

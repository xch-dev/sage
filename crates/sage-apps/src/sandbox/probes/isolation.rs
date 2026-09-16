use std::time::Duration;

use serde::Deserialize;
use tauri::{AppHandle, Listener, State};
use tokio::sync::oneshot;
use tokio::time::timeout;

use super::super::runtime::{start_test_app, stop_test_apps, unique_run_id};
use super::poll::poll_isolation;
use crate::{
    AppsHostState, BUILTIN_STORAGE_ISOLATION_INCOGNITO_ID, BUILTIN_STORAGE_ISOLATION_PERSISTENT_ID,
    get_sage_webview,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct HostIsolationSeedResult {
    run_id: String,
    local_storage_seeded: bool,
    indexed_db_seeded: bool,
    error: Option<String>,
}

async fn seed_host_isolation_probe(app: &AppHandle, run_id: &str) -> Result<(), String> {
    let webview = get_sage_webview(app)?;
    let event_name = format!("sage:sandbox-host-seed:{run_id}");
    let event_name_json = serde_json::to_string(&event_name)
        .map_err(|err| format!("failed to encode isolation seed event name: {err}"))?;
    let run_id_json = serde_json::to_string(run_id)
        .map_err(|err| format!("failed to encode isolation probe value: {err}"))?;

    let (sender, receiver) = oneshot::channel();
    let listener_id = webview.once(event_name.clone(), move |event| {
        let _ = sender.send(event.payload().to_string());
    });

    let script = format!(
        r"(async function () {{
  const eventName = {event_name_json};
  const runId = {run_id_json};
  const result = {{
    runId,
    localStorageSeeded: false,
    indexedDbSeeded: false,
    error: null,
  }};

  function errorMessage(error) {{
    return error instanceof Error ? error.message : String(error);
  }}

  function seedIndexedDb() {{
    return new Promise((resolve, reject) => {{
      let db = null;
      let settled = false;

      function closeDb() {{
        try {{ db?.close(); }} catch {{}}
      }}

      function fail(error) {{
        if (settled) return;
        settled = true;
        closeDb();
        reject(error);
      }}

      function succeed(value) {{
        if (settled) return;
        settled = true;
        closeDb();
        resolve(value);
      }}

      let open;
      try {{
        open = indexedDB.open('sage_probe_db', 1);
      }} catch (error) {{
        fail(error);
        return;
      }}

      open.onerror = () => fail(open.error ?? new Error('IndexedDB open failed'));
      open.onblocked = () => fail(new Error('IndexedDB open was blocked'));
      open.onupgradeneeded = () => {{
        try {{
          const upgradeDb = open.result;
          if (!upgradeDb.objectStoreNames.contains('probe_store')) {{
            upgradeDb.createObjectStore('probe_store');
          }}
        }} catch (error) {{
          try {{ open.transaction?.abort(); }} catch {{}}
          fail(error);
        }}
      }};
      open.onsuccess = () => {{
        try {{
          db = open.result;
          if (!db.objectStoreNames.contains('probe_store')) {{
            fail(new Error('IndexedDB probe store is missing'));
            return;
          }}

          const writeTx = db.transaction('probe_store', 'readwrite');
          writeTx.objectStore('probe_store').put(runId, 'sage_probe_key');
          writeTx.onerror = () =>
            fail(writeTx.error ?? new Error('IndexedDB probe write failed'));
          writeTx.onabort = () =>
            fail(writeTx.error ?? new Error('IndexedDB probe write was aborted'));
          writeTx.oncomplete = () => {{
            try {{
              const readTx = db.transaction('probe_store', 'readonly');
              const read = readTx.objectStore('probe_store').get('sage_probe_key');
              read.onerror = () =>
                fail(read.error ?? new Error('IndexedDB probe verification failed'));
              read.onsuccess = () => succeed(read.result === runId);
            }} catch (error) {{
              fail(error);
            }}
          }};
        }} catch (error) {{
          fail(error);
        }}
      }};
    }});
  }}

  try {{
    localStorage.setItem('sage_probe_local_storage', runId);
    result.localStorageSeeded =
      localStorage.getItem('sage_probe_local_storage') === runId;
    if (!result.localStorageSeeded) {{
      throw new Error('localStorage probe verification failed');
    }}

    result.indexedDbSeeded = await seedIndexedDb();
    if (!result.indexedDbSeeded) {{
      throw new Error('IndexedDB probe verification failed');
    }}
  }} catch (error) {{
    result.error = errorMessage(error);
  }}

  await window.__TAURI__.event.emit(eventName, result);
}})();"
    );

    if let Err(err) = webview.eval(script) {
        webview.unlisten(listener_id);
        return Err(format!("failed to seed host isolation probe data: {err}"));
    }

    let payload = match timeout(Duration::from_secs(5), receiver).await {
        Ok(Ok(payload)) => payload,
        Ok(Err(_)) => {
            return Err("host isolation seed acknowledgement channel closed".into());
        }
        Err(_) => {
            webview.unlisten(listener_id);
            return Err("timed out waiting for host isolation seed acknowledgement".into());
        }
    };

    let result: HostIsolationSeedResult = serde_json::from_str(&payload)
        .map_err(|err| format!("invalid host isolation seed acknowledgement: {err}"))?;

    if result.run_id != run_id {
        return Err("host isolation seed acknowledgement had the wrong run ID".into());
    }

    if let Some(error) = result.error {
        return Err(format!("failed to seed host isolation probe data: {error}"));
    }

    if !result.local_storage_seeded || !result.indexed_db_seeded {
        return Err("host isolation probe data could not be verified".into());
    }

    Ok(())
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

    stop_test_apps(app, apps_state, &app_ids).await?;

    seed_host_isolation_probe(app, &run_id).await?;

    let probe_result = async {
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

        poll_isolation(apps_state, &run_id, 2, 2_000).await
    }
    .await;

    let stop_result = stop_test_apps(app, apps_state, &app_ids).await;
    let results = probe_result?;
    stop_result?;

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

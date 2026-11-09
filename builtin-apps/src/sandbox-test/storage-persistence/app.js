import './bridge.js';
import { getSageClient } from './sdk.js';

const log = (...args) => window.__SAGE_TEST__?.log?.(...args);

(async () => {
  log('start', window.location.href);

  const sage = await getSageClient();
  log('getSageClient ok');

  const ping = await sage.app.bridgePing();
  log('bridgePing ok', ping);

  const params = new URLSearchParams(window.location.search);
  const runId = params.get('runId');
  const phase = params.get('phase');

  if (!runId) {
    throw new Error('missing runId');
  }

  if (phase !== 'write' && phase !== 'read') {
    throw new Error('missing or invalid phase');
  }

  const LOCAL_STORAGE_KEY = `sandbox_persistence_local_storage:${runId}:incognito`;
  const DB_NAME = `sandbox_persistence_db_${runId}_incognito`;
  const STORE_NAME = 'probe_store';
  const DB_KEY = 'probe_key';

  async function writeIndexedDbValue() {
    try {
      return await new Promise((resolve) => {
        const open = indexedDB.open(DB_NAME);

        open.onerror = () => resolve(false);

        open.onupgradeneeded = () => {
          try {
            const db = open.result;
            if (!db.objectStoreNames.contains(STORE_NAME)) {
              db.createObjectStore(STORE_NAME);
            }
          } catch {}
        };

        open.onsuccess = () => {
          try {
            const db = open.result;
            const tx = db.transaction(STORE_NAME, 'readwrite');
            const store = tx.objectStore(STORE_NAME);
            const req = store.put('present', DB_KEY);

            req.onerror = () => {
              db.close();
              resolve(false);
            };

            req.onsuccess = () => {
              tx.oncomplete = () => {
                db.close();
                resolve(true);
              };

              tx.onerror = () => {
                db.close();
                resolve(false);
              };
            };
          } catch {
            resolve(false);
          }
        };
      });
    } catch {
      return false;
    }
  }

  async function readIndexedDbValue() {
    try {
      return await new Promise((resolve) => {
        const open = indexedDB.open(DB_NAME);

        open.onerror = () => resolve(false);

        open.onupgradeneeded = () => {
          try {
            const db = open.result;
            if (!db.objectStoreNames.contains(STORE_NAME)) {
              db.createObjectStore(STORE_NAME);
            }
          } catch {}
        };

        open.onsuccess = () => {
          try {
            const db = open.result;

            if (!db.objectStoreNames.contains(STORE_NAME)) {
              db.close();
              resolve(false);
              return;
            }

            const tx = db.transaction(STORE_NAME, 'readonly');
            const store = tx.objectStore(STORE_NAME);
            const req = store.get(DB_KEY);

            req.onerror = () => {
              db.close();
              resolve(false);
            };

            req.onsuccess = () => {
              db.close();
              resolve(req.result === 'present');
            };
          } catch {
            resolve(false);
          }
        };
      });
    } catch {
      return false;
    }
  }

  async function reportWrite(data) {
    log('bridgeSend persistence_write start', data);
    const result = await sage.app.bridgeSend({
      kind: 'sandbox_report',
      report: {
        type: 'persistence_write',
        data,
      },
    });
    log('bridgeSend persistence_write ok', result);
  }

  async function reportRead(data) {
    log('bridgeSend persistence_read start', data);
    const result = await sage.app.bridgeSend({
      kind: 'sandbox_report',
      report: {
        type: 'persistence_read',
        data,
      },
    });
    log('bridgeSend persistence_read ok', result);
  }

  if (phase === 'write') {
    let localStorageWrote = false;
    let indexedDbWrote = false;
    let error = null;

    try {
      try {
        localStorage.setItem(LOCAL_STORAGE_KEY, 'present');
        localStorageWrote =
          localStorage.getItem(LOCAL_STORAGE_KEY) === 'present';
        log('localStorageWrote', localStorageWrote);
      } catch {
        localStorageWrote = false;
        log('localStorage write failed');
      }

      indexedDbWrote = await writeIndexedDbValue();
      log('indexedDbWrote', indexedDbWrote);
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
      log('write phase error', error);
    }

    await reportWrite({
      runId,
      localStorageWrote,
      indexedDbWrote,
      error,
    });

    return;
  }

  let localStoragePresent = false;
  let indexedDbPresent = false;
  let error = null;

  try {
    try {
      localStoragePresent =
        localStorage.getItem(LOCAL_STORAGE_KEY) === 'present';
      log('localStoragePresent', localStoragePresent);
    } catch {
      localStoragePresent = false;
      log('localStorage read failed');
    }

    indexedDbPresent = await readIndexedDbValue();
    log('indexedDbPresent', indexedDbPresent);
  } catch (err) {
    error = err instanceof Error ? err.message : String(err);
    log('read phase error', error);
  }

  await reportRead({
    runId,
    localStoragePresent,
    indexedDbPresent,
    error,
  });
})().catch(async (err) => {
  log('fatal', err instanceof Error ? err.message : String(err));

  try {
    const sage = await getSageClient();
    const params = new URLSearchParams(window.location.search);
    const phase = params.get('phase');

    if (phase === 'write') {
      const payload = {
        runId: params.get('runId'),
        localStorageWrote: false,
        indexedDbWrote: false,
        error: err instanceof Error ? err.message : String(err),
      };

      log('fallback bridgeSend persistence_write start', payload);

      const result = await sage.app.bridgeSend({
        kind: 'sandbox_report',
        report: {
          type: 'persistence_write',
          data: payload,
        },
      });

      log('fallback bridgeSend persistence_write ok', result);
      return;
    }

    const payload = {
      runId: params.get('runId'),
      localStoragePresent: false,
      indexedDbPresent: false,
      error: err instanceof Error ? err.message : String(err),
    };

    log('fallback bridgeSend persistence_read start', payload);

    const result = await sage.app.bridgeSend({
      kind: 'sandbox_report',
      report: {
        type: 'persistence_read',
        data: payload,
      },
    });

    log('fallback bridgeSend persistence_read ok', result);
  } catch (fallbackErr) {
    log(
      'fallback failed',
      fallbackErr instanceof Error ? fallbackErr.message : String(fallbackErr),
    );
  }
});

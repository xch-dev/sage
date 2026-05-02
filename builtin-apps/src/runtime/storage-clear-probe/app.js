import './bridge.js';
import { getSageClient } from './sdk.js';

const log = (...args) => console.log('[storage-clear-probe]', ...args);

const STORE_NAME = 'probe_store';
const DB_KEY = 'probe_key';

function getKeys(runId) {
  return {
    localStorageKey: `storage_clear_probe_local_storage:${runId}`,
    dbName: `storage_clear_probe_db_${runId}`,
  };
}

async function writeIndexedDbValue(dbName) {
  try {
    return await new Promise((resolve) => {
      const open = indexedDB.open(dbName);

      open.onerror = () => resolve(false);

      open.onupgradeneeded = () => {
        try {
          const db = open.result;
          if (!db.objectStoreNames.contains(STORE_NAME)) {
            db.createObjectStore(STORE_NAME);
          }
        } catch {
          //
        }
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

async function readIndexedDbValue(dbName) {
  try {
    return await new Promise((resolve) => {
      const open = indexedDB.open(dbName);

      open.onerror = () => resolve(false);

      open.onupgradeneeded = () => {
        try {
          const db = open.result;
          if (!db.objectStoreNames.contains(STORE_NAME)) {
            db.createObjectStore(STORE_NAME);
          }
        } catch {
          //
        }
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

async function report(sage, data) {
  log('bridgeSend clear_cycle start', data);
  const result = await sage.app.bridgeSend({
    kind: 'sandbox_report',
    report: {
      type: 'clear_cycle',
      data,
    },
  });
  log('bridgeSend clear_cycle ok', result);
}

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

  if (
    phase !== 'write' &&
    phase !== 'check_present' &&
    phase !== 'check_absent'
  ) {
    throw new Error('missing or invalid phase');
  }

  const { localStorageKey, dbName } = getKeys(runId);

  let localStoragePresent = false;
  let indexedDbPresent = false;
  let error = null;

  try {
    if (phase === 'write') {
      try {
        localStorage.setItem(localStorageKey, 'present');
        log('localStorage write attempted', localStorageKey);
      } catch {
        log('localStorage write failed', localStorageKey);
      }

      const wrote = await writeIndexedDbValue(dbName);
      log('indexedDb write result', wrote, dbName);
    }

    try {
      localStoragePresent = localStorage.getItem(localStorageKey) === 'present';
      log('localStoragePresent', localStoragePresent);
    } catch {
      localStoragePresent = false;
      log('localStorage read failed');
    }

    indexedDbPresent = await readIndexedDbValue(dbName);
    log('indexedDbPresent', indexedDbPresent);
  } catch (err) {
    error = err instanceof Error ? err.message : String(err);
    log('probe error', error);
  }

  await report(sage, {
    runId,
    phase,
    localStoragePresent,
    indexedDbPresent,
    error,
  });
})().catch(async (err) => {
  log('fatal', err instanceof Error ? err.message : String(err));

  try {
    const sage = await getSageClient();
    const params = new URLSearchParams(window.location.search);

    await report(sage, {
      runId: params.get('runId'),
      phase: params.get('phase'),
      localStoragePresent: false,
      indexedDbPresent: false,
      error: err instanceof Error ? err.message : String(err),
    });
  } catch (fallbackErr) {
    log(
      'fallback failed',
      fallbackErr instanceof Error ? fallbackErr.message : String(fallbackErr),
    );
  }
});

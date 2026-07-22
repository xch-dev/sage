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

  if (!runId) {
    throw new Error('missing runId');
  }

  const LOCAL_STORAGE_KEY = 'sage_probe_local_storage';
  const DB_NAME = 'sage_probe_db';
  const STORE_NAME = 'probe_store';
  const DB_KEY = 'sage_probe_key';

  async function readIndexedDbProbe(expectedValue) {
    return await new Promise((resolve, reject) => {
      let db = null;

      try {
        const open = indexedDB.open(DB_NAME);

        open.onerror = () =>
          reject(open.error ?? new Error('IndexedDB open failed'));
        open.onblocked = () => reject(new Error('IndexedDB open was blocked'));

        open.onupgradeneeded = () => {
          try {
            const upgradeDb = open.result;
            if (!upgradeDb.objectStoreNames.contains(STORE_NAME)) {
              upgradeDb.createObjectStore(STORE_NAME);
            }
          } catch (error) {
            try {
              open.transaction?.abort();
            } catch {}
            reject(error);
          }
        };

        open.onsuccess = () => {
          try {
            db = open.result;

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
              reject(req.error ?? new Error('IndexedDB probe read failed'));
            };

            req.onsuccess = () => {
              db.close();
              resolve(req.result === expectedValue);
            };
          } catch (error) {
            try {
              db?.close();
            } catch {}
            reject(error);
          }
        };
      } catch (error) {
        try {
          db?.close();
        } catch {}
        reject(error);
      }
    });
  }

  async function report(data) {
    log('bridgeSend isolation start', data);
    const result = await sage.app.bridgeSend({
      kind: 'sandbox_report',
      report: {
        type: 'isolation',
        data,
      },
    });
    log('bridgeSend isolation ok', result);
  }

  let localStorageVisible = false;
  let indexedDbVisible = false;
  let error = null;

  try {
    const value = localStorage.getItem(LOCAL_STORAGE_KEY);
    localStorageVisible = value === runId;
    log('localStorageVisible', localStorageVisible);

    indexedDbVisible = await readIndexedDbProbe(runId);
    log('indexedDbVisible', indexedDbVisible);
  } catch (err) {
    error = err instanceof Error ? err.message : String(err);
    log('probe error', error);
  }

  await report({
    runId,
    localStorageVisible,
    indexedDbVisible,
    error,
  });
})().catch(async (err) => {
  log('fatal', err instanceof Error ? err.message : String(err));

  try {
    const sage = await getSageClient();
    const params = new URLSearchParams(window.location.search);

    const payload = {
      runId: params.get('runId'),
      localStorageVisible: false,
      indexedDbVisible: false,
      error: err instanceof Error ? err.message : String(err),
    };

    log('fallback bridgeSend isolation start', payload);

    const result = await sage.app.bridgeSend({
      kind: 'sandbox_report',
      report: {
        type: 'isolation',
        data: payload,
      },
    });

    log('fallback bridgeSend isolation ok', result);
  } catch (fallbackErr) {
    log(
      'fallback failed',
      fallbackErr instanceof Error ? fallbackErr.message : String(fallbackErr),
    );
  }
});

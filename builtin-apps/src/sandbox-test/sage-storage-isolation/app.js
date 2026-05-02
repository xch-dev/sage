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

  async function readIndexedDbProbe() {
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
              resolve(typeof req.result === 'string' && req.result.length > 0);
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
    try {
      const value = localStorage.getItem(LOCAL_STORAGE_KEY);
      localStorageVisible = typeof value === 'string' && value.length > 0;
      log('localStorageVisible', localStorageVisible);
    } catch {
      localStorageVisible = false;
      log('localStorage read failed');
    }

    indexedDbVisible = await readIndexedDbProbe();
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

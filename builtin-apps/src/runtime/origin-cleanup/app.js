import './bridge.js';
import { getSageClient } from './sdk.js';

const log = (...args) => window.__SAGE_TEST__?.log?.(...args);

async function deleteIndexedDb(name) {
  return await new Promise((resolve) => {
    const req = indexedDB.deleteDatabase(name);

    req.onsuccess = () => resolve(null);
    req.onerror = () => resolve(`indexedDB ${name}: ${req.error}`);
    req.onblocked = () => resolve(`indexedDB ${name}: blocked`);
  });
}

async function clearOriginData() {
  const errors = [];

  try {
    localStorage.clear();
  } catch (err) {
    errors.push(`localStorage: ${String(err)}`);
  }

  try {
    sessionStorage.clear();
  } catch (err) {
    errors.push(`sessionStorage: ${String(err)}`);
  }

  try {
    if ('caches' in window) {
      const keys = await caches.keys();
      await Promise.all(keys.map((key) => caches.delete(key)));
    }
  } catch (err) {
    errors.push(`caches: ${String(err)}`);
  }

  try {
    if (indexedDB.databases) {
      const dbs = await indexedDB.databases();

      for (const db of dbs) {
        if (!db.name) continue;

        const error = await deleteIndexedDb(db.name);
        if (error) errors.push(error);
      }
    } else {
      errors.push('indexedDB.databases unavailable');
    }
  } catch (err) {
    errors.push(`indexedDB: ${String(err)}`);
  }

  try {
    for (const cookie of document.cookie.split(';')) {
      const name = cookie.split('=')[0]?.trim();
      if (!name) continue;

      document.cookie = `${name}=; Max-Age=0; path=/`;
      document.cookie = `${name}=; expires=Thu, 01 Jan 1970 00:00:00 GMT; path=/`;
    }
  } catch (err) {
    errors.push(`cookies: ${String(err)}`);
  }

  return errors;
}

async function report(sage, cleanupId, ok, errors) {
  const payload = {
    kind: 'originCleanup.completed',
    cleanupId,
    ok,
    errors,
  };

  log('bridgeSend originCleanup.completed start', payload);

  const result = await sage.app.bridgeSend(payload);

  log('bridgeSend originCleanup.completed ok', result);
}

(async () => {
  log('start', window.location.href);

  const sage = await getSageClient();
  log('getSageClient ok');

  const ping = await sage.app.bridgePing();
  log('bridgePing ok', ping);

  const params = new URLSearchParams(window.location.search);
  const cleanupId = params.get('cleanupId');

  if (!cleanupId) {
    throw new Error('missing cleanupId');
  }

  const errors = await clearOriginData();

  await report(sage, cleanupId, errors.length === 0, errors);
})().catch(async (err) => {
  log('fatal', err instanceof Error ? err.message : String(err));

  try {
    const sage = await getSageClient();
    const params = new URLSearchParams(window.location.search);
    const cleanupId = params.get('cleanupId') ?? '';

    await report(sage, cleanupId, false, [
      err instanceof Error ? err.message : String(err),
    ]);
  } catch (fallbackErr) {
    log(
      'fallback failed',
      fallbackErr instanceof Error ? fallbackErr.message : String(fallbackErr),
    );
  }
});

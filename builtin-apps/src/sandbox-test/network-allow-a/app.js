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

  const allowedUrl = 'https://example.com/';
  const blockedUrl = 'https://example.org/';

  async function tryFetch(url) {
    const controller = new AbortController();
    const timeout = window.setTimeout(() => controller.abort(), 5000);

    try {
      await fetch(url, {
        method: 'GET',
        mode: 'no-cors',
        cache: 'no-store',
        signal: controller.signal,
      });

      window.clearTimeout(timeout);
      return true;
    } catch {
      window.clearTimeout(timeout);
      return false;
    }
  }

  let allowedOk = false;
  let blockedOk = false;
  let error = null;

  try {
    allowedOk = await tryFetch(allowedUrl);
    log('allowedOk', allowedUrl, allowedOk);

    blockedOk = await tryFetch(blockedUrl);
    log('blockedOk', blockedUrl, blockedOk);
  } catch (err) {
    error = err instanceof Error ? err.message : String(err);
    log('network probe error', error);
  }

  const payload = {
    runId,
    mode: 'allow-a',
    allowedUrl,
    blockedUrl,
    allowedOk,
    blockedOk,
    error,
  };

  log('bridgeSend network start', payload);

  const result = await sage.app.bridgeSend({
    kind: 'sandbox_report',
    report: {
      type: 'network',
      data: payload,
    },
  });

  log('bridgeSend network ok', result);
})().catch(async (err) => {
  log('fatal', err instanceof Error ? err.message : String(err));

  try {
    const sage = await getSageClient();
    const params = new URLSearchParams(window.location.search);

    const payload = {
      runId: params.get('runId'),
      mode: 'allow-a',
      allowedUrl: 'https://example.com/',
      blockedUrl: 'https://example.org/',
      allowedOk: false,
      blockedOk: false,
      error: err instanceof Error ? err.message : String(err),
    };

    log('fallback bridgeSend network start', payload);

    const result = await sage.app.bridgeSend({
      kind: 'sandbox_report',
      report: {
        type: 'network',
        data: payload,
      },
    });

    log('fallback bridgeSend network ok', result);
  } catch (fallbackErr) {
    log(
      'fallback failed',
      fallbackErr instanceof Error ? fallbackErr.message : String(fallbackErr),
    );
  }
});

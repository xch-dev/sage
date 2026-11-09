(function () {
  function ensureRoot() {
    let root = document.getElementById('__sage_test_debug_root');
    if (root) return root;

    root = document.createElement('div');
    root.id = '__sage_test_debug_root';
    Object.assign(root.style, {
      position: 'fixed',
      top: '6px',
      left: '6px',
      right: '6px',
      bottom: '6px',
      zIndex: '2147483647',
      pointerEvents: 'none',
      display: 'flex',
      flexDirection: 'column',
      gap: '6px',
      fontFamily: 'monospace',
      minHeight: '0',
      minWidth: '0',
      boxSizing: 'border-box',
    });

    document.body.appendChild(root);
    return root;
  }

  function mountDebugLabel() {
    const params = new URLSearchParams(window.location.search);
    const appId = window.location.host || 'unknown-app';
    const phase = params.get('phase');
    const runId = params.get('runId');

    const parts = [appId];
    if (phase) parts.push(phase);
    if (runId) parts.push(runId);

    const el = document.createElement('div');
    el.textContent = parts.join(' • ');

    Object.assign(el.style, {
      alignSelf: 'stretch',
      flex: '0 0 auto',
      pointerEvents: 'none',
      padding: '3px 6px',
      borderRadius: '4px',
      background: 'rgba(0,0,0,0.72)',
      color: '#00ff88',
      fontSize: '10px',
      lineHeight: '1.2',
      whiteSpace: 'normal',
      overflowWrap: 'anywhere',
      wordBreak: 'break-word',
      maxWidth: '100%',
      boxSizing: 'border-box',
    });

    ensureRoot().appendChild(el);
    return el;
  }

  function ensureLogPane() {
    let pane = document.getElementById('__sage_test_log_pane');
    if (pane) return pane;

    pane = document.createElement('div');
    pane.id = '__sage_test_log_pane';

    Object.assign(pane.style, {
      flex: '1 1 auto',
      minHeight: '0',
      minWidth: '0',
      pointerEvents: 'auto',
      overflowY: 'auto',
      overflowX: 'hidden',
      padding: '6px',
      borderRadius: '4px',
      background: 'rgba(0,0,0,0.72)',
      color: '#e5e7eb',
      fontSize: '10px',
      lineHeight: '1.35',
      whiteSpace: 'pre-wrap',
      overflowWrap: 'anywhere',
      wordBreak: 'break-word',
      boxSizing: 'border-box',
      overscrollBehavior: 'contain',
    });

    ensureRoot().appendChild(pane);
    return pane;
  }

  function stringify(value) {
    if (typeof value === 'string') return value;
    try {
      return JSON.stringify(value, null, 2);
    } catch {
      return String(value);
    }
  }

  function log(...args) {
    const pane = ensureLogPane();
    const line = document.createElement('div');
    line.textContent = args.map(stringify).join(' ');
    pane.appendChild(line);
    pane.scrollTop = pane.scrollHeight;
  }

  function patchConsole() {
    const orig = {
      log: console.log.bind(console),
      warn: console.warn.bind(console),
      error: console.error.bind(console),
    };

    console.log = (...args) => {
      log('[log]', ...args);
      orig.log(...args);
    };

    console.warn = (...args) => {
      log('[warn]', ...args);
      orig.warn(...args);
    };

    console.error = (...args) => {
      log('[error]', ...args);
      orig.error(...args);
    };

    window.addEventListener('error', (event) => {
      log('[window.error]', event.message);
    });

    window.addEventListener('unhandledrejection', (event) => {
      log('[unhandledrejection]', event.reason);
    });
  }

  window.__SAGE_TEST__ = {
    ...(window.__SAGE_TEST__ || {}),
    mountDebugLabel,
    ensureLogPane,
    log,
  };

  mountDebugLabel();
  ensureLogPane();
  patchConsole();
})();

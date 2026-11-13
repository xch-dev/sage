/**
 * Runs Sage's UI in a normal browser against the sandbox backend from `pnpm sandbox`.
 *
 * Vite only injects this script in sandbox mode, ahead of `main.tsx`, so it never ships in a build.
 * It replaces Tauri's IPC with HTTP calls to the sandbox bridge and gives the plugins the UI uses
 * browser stand-ins. See `docs/sandbox.md`.
 */

import { mockIPC, mockWindows } from '@tauri-apps/api/mocks';

declare const __SAGE_SANDBOX_VERSION__: string;

declare global {
  interface Window {
    /** Inspection hooks for agents driving the sandbox through a browser. */
    __SAGE_SANDBOX__?: SandboxGlobals;
  }
}

interface SandboxGlobals {
  /** The wallet this tab is connected to, such as `alice` or `bob`. */
  wallet: string;
  /** The last text the app copied, since the browser clipboard may not be readable. */
  clipboard: string | null;
  /** URLs the app asked to open externally, oldest first. */
  openedUrls: string[];
  /** Commands that failed because the sandbox has no stand-in for them. */
  unsupported: string[];
}

/** Picks the wallet from the host, so `bob.localhost` talks to bob and anything else to alice. */
function walletFromHost(): string {
  const [first, ...rest] = window.location.hostname.split('.');
  return rest.length > 0 && rest[rest.length - 1] === 'localhost'
    ? first
    : 'alice';
}

function osPlatform(): 'macos' | 'windows' | 'linux' {
  const agent = navigator.userAgent;
  if (agent.includes('Mac')) return 'macos';
  if (agent.includes('Windows')) return 'windows';
  return 'linux';
}

const sandbox: SandboxGlobals = {
  wallet: walletFromHost(),
  clipboard: null,
  openedUrls: [],
  unsupported: [],
};

window.__SAGE_SANDBOX__ = sandbox;

const platform = osPlatform();

window.__TAURI_OS_PLUGIN_INTERNALS__ = {
  platform,
  os_type: platform,
  family: platform === 'windows' ? 'windows' : 'unix',
  eol: platform === 'windows' ? '\r\n' : '\n',
  version: '',
  // The UI doesn't read the architecture, but the plugin's type requires one.
  arch: 'x86_64',
  exe_extension: platform === 'windows' ? 'exe' : '',
};

const bridgeUrl = `/__sandbox/${sandbox.wallet}`;

function unsupported(cmd: string): never {
  sandbox.unsupported.push(cmd);
  console.warn(`[sandbox] ${cmd} isn't available in the sandbox`);
  // Match the shape of the desktop app's command errors, which the UI reads `reason` from.
  throw { kind: 'api', reason: `${cmd} isn't available in the sandbox` };
}

async function callBridge(cmd: string, args: unknown): Promise<unknown> {
  const response = await fetch(`${bridgeUrl}/invoke/${cmd}`, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify(args ?? {}),
  });

  const text = await response.text();
  const body = text ? JSON.parse(text) : null;

  // The desktop app rejects with the error object itself, not an `Error`.
  if (!response.ok) throw body;

  return body;
}

/**
 * Stand-ins for the parts of the mini-apps host the main window reads at startup. Apps themselves
 * run in native webviews, so they can't be opened from a browser.
 */
function appsCommand(cmd: string): unknown {
  switch (cmd) {
    case 'apps_enter_workspace':
    case 'apps_leave_workspace':
    case 'apps_set_environment_theme':
    case 'apps_clear_active_taskbar_runtime':
      return null;
    case 'apps_list_installed_apps':
    case 'apps_list_runtimes':
      return [];
    case 'apps_get_auto_update_enabled':
      return false;
    default:
      return unsupported(cmd);
  }
}

async function pluginCommand(
  cmd: string,
  args: Record<string, unknown>,
): Promise<unknown> {
  switch (cmd) {
    case 'plugin:app|version':
      return __SAGE_SANDBOX_VERSION__;
    case 'plugin:app|name':
      return 'Sage';
    case 'plugin:clipboard-manager|write_text': {
      const text = String(args.text ?? '');
      sandbox.clipboard = text;
      // The page may not be allowed to write the clipboard, which `sandbox.clipboard` covers.
      await navigator.clipboard?.writeText(text).catch(() => undefined);
      return null;
    }
    case 'plugin:clipboard-manager|read_text':
      return (
        (await navigator.clipboard?.readText().catch(() => null)) ??
        sandbox.clipboard ??
        ''
      );
    case 'plugin:opener|open_url': {
      const url = String(args.url);
      sandbox.openedUrls.push(url);
      window.open(url, '_blank', 'noopener');
      return null;
    }
  }

  // Window calls such as focusing or requesting attention have nothing to act on in a tab.
  if (cmd.startsWith('plugin:window|')) return null;

  return unsupported(cmd);
}

/**
 * Callback IDs listening to each event. This stands in for Tauri's event plugin, since the one in
 * `@tauri-apps/api/mocks` never removes a listener on `unlisten`.
 */
const listeners = new Map<string, number[]>();

function runCallback(id: number, data: unknown) {
  // `mockIPC` installs this on `window.__TAURI_INTERNALS__`, which has no global type.
  const { __TAURI_INTERNALS__: internals } = window as unknown as {
    __TAURI_INTERNALS__: { runCallback: (id: number, data: unknown) => void };
  };
  internals.runCallback(id, data);
}

function emitEvent(event: string, payload: unknown) {
  for (const id of listeners.get(event) ?? []) {
    runCallback(id, { event, id, payload });
  }
}

function eventCommand(cmd: string, args: Record<string, unknown>): unknown {
  const event = String(args.event);

  switch (cmd) {
    case 'plugin:event|listen': {
      // Tauri answers with an event ID that `unlisten` passes back. The callback ID is unique too.
      const handler = Number(args.handler);
      listeners.set(event, [...(listeners.get(event) ?? []), handler]);
      return handler;
    }
    case 'plugin:event|unlisten':
      listeners.set(
        event,
        (listeners.get(event) ?? []).filter((id) => id !== args.eventId),
      );
      return null;
    case 'plugin:event|emit':
      emitEvent(event, args.payload);
      return null;
    default:
      return unsupported(cmd);
  }
}

mockWindows('main');

mockIPC((cmd, args) => {
  const payload = (args ?? {}) as Record<string, unknown>;

  if (cmd.startsWith('plugin:event|')) return eventCommand(cmd, payload);
  if (cmd.startsWith('plugin:')) return pluginCommand(cmd, payload);
  if (cmd.startsWith('apps_')) return appsCommand(cmd);

  return callBridge(cmd, payload);
});

// Forward the wallet's sync events, which the app listens for as `sync-event`.
const events = new EventSource(`${bridgeUrl}/events`);

events.onmessage = (message) => {
  emitEvent('sync-event', JSON.parse(message.data));
};

document.title = `Sage sandbox (${sandbox.wallet})`;

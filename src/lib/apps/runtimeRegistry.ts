import { Webview } from '@tauri-apps/api/webview';
import {
  commands,
  type CreateRuntimeArgs,
  type SageAppRuntimeRecordView,
  type SystemSageAppView,
  type UserSageAppView,
  type SageAppView,
  type RuntimeTargetParams,
} from '@/bindings';

export type { SageAppRuntimeRecordView };

type RuntimeListener = (records: SageAppRuntimeRecordView[]) => void;
type AppLike = SageAppView | UserSageAppView | SystemSageAppView;

const listeners = new Set<RuntimeListener>();
let cachedRuntimes: SageAppRuntimeRecordView[] = [];
let pollTimer: number | null = null;
let polling = false;

async function refreshRuntimes(): Promise<SageAppRuntimeRecordView[]> {
  if (polling) {
    return cachedRuntimes;
  }

  polling = true;
  try {
    const next = await commands.appsListRuntimes();
    cachedRuntimes = next;

    for (const listener of listeners) {
      listener(next);
    }

    return next;
  } catch (err) {
    console.error('Failed to refresh app runtimes:', err);
    return cachedRuntimes;
  } finally {
    polling = false;
  }
}

function ensurePolling() {
  if (pollTimer != null) {
    return;
  }

  void refreshRuntimes();

  pollTimer = window.setInterval(() => {
    void refreshRuntimes();
  }, 1000);
}

function maybeStopPolling() {
  if (listeners.size > 0) {
    return;
  }

  if (pollTimer != null) {
    window.clearInterval(pollTimer);
    pollTimer = null;
  }
}

function runtimeTarget(appId: string): RuntimeTargetParams {
  return { appId };
}

export function subscribeAppRuntimes(listener: RuntimeListener): () => void {
  listeners.add(listener);
  listener(cachedRuntimes);
  ensurePolling();

  return () => {
    listeners.delete(listener);
    maybeStopPolling();
  };
}

export function listAppRuntimes(): SageAppRuntimeRecordView[] {
  return cachedRuntimes;
}

export async function getRuntimeWebview(
  appId: string,
): Promise<Webview | null> {
  const runtime =
    cachedRuntimes.find((item) => item.app.common.identity.id === appId) ??
    (await refreshRuntimes()).find(
      (item) => item.app.common.identity.id === appId,
    );

  if (!runtime) {
    return null;
  }

  return await Webview.getByLabel(runtime.webviewLabel).catch(() => null);
}

export async function markRuntimeVisible(
  appId: string,
  visible: boolean,
): Promise<void> {
  if (visible) {
    await commands.appsFocusRuntime(runtimeTarget(appId));
  } else {
    await commands.appsHideRuntime(runtimeTarget(appId));
  }

  await refreshRuntimes();
}

export async function focusRuntime(appId: string): Promise<void> {
  await commands.appsFocusRuntime(runtimeTarget(appId));
  await refreshRuntimes();
}

export async function hideRuntime(appId: string): Promise<void> {
  await commands.appsHideRuntime(runtimeTarget(appId));
  await refreshRuntimes();
}

export async function killRuntime(appId: string): Promise<void> {
  await commands.appsKillRuntime(runtimeTarget(appId));
  await refreshRuntimes();
}

export async function closeAppRuntime(
  appId: string,
  options?: { timeoutMs?: number },
): Promise<void> {
  void options;
  await killRuntime(appId);
}

export async function ensureInlineRuntime(
  app: AppLike,
): Promise<SageAppRuntimeRecordView> {
  const args: CreateRuntimeArgs = {
    appId: app.common.identity.id,
    mode: 'Inline',
    visibility: 'Visible',
    debugLayout: false,
    query: {},
  };

  const created = await commands.appsCreateInlineRuntime(args);
  await refreshRuntimes();
  return created;
}

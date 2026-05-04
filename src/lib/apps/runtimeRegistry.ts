import { Webview, getCurrentWebview } from '@tauri-apps/api/webview';
import {
  commands,
  type CreateInstalledRuntimeArgs,
  type SageAppRuntimeRecordView,
  type SystemSageAppView,
  type UserSageAppView,
  type SageAppView,
  type RuntimeTargetParams,
} from '@/bindings';

export type { SageAppRuntimeRecordView };

type RuntimeListener = (records: SageAppRuntimeRecordView[]) => void;
type ActiveRuntimeListener = (event: ActiveRuntimeChangedPayload) => void;
type AppLike = SageAppView | UserSageAppView | SystemSageAppView;

interface RuntimeEventEnvelope<T = unknown> {
  type: string;
  payload: T;
}

interface RuntimesChangedPayload {
  runtimes: SageAppRuntimeRecordView[];
}

export interface ActiveRuntimeChangedPayload {
  hostWindowLabel: string;
  appId: string | null;
  runtimeId: string | null;
}

const HOST_RUNTIME_EVENT_NAME = 'apps:runtime-event';

const listeners = new Set<RuntimeListener>();
const activeRuntimeListeners = new Set<ActiveRuntimeListener>();

let cachedRuntimes: SageAppRuntimeRecordView[] = [];
let pollTimer: number | null = null;
let polling = false;

let runtimeEventsMounted = false;
let unlistenRuntimeEvents: (() => void) | null = null;

function notifyRuntimeListeners(next: SageAppRuntimeRecordView[]) {
  cachedRuntimes = next;

  for (const listener of listeners) {
    listener(next);
  }
}

function notifyActiveRuntimeListeners(event: ActiveRuntimeChangedPayload) {
  for (const listener of activeRuntimeListeners) {
    listener(event);
  }
}

function isObject(value: unknown): value is Record<string, unknown> {
  return !!value && typeof value === 'object';
}

function isRuntimesChangedEvent(
  value: unknown,
): value is RuntimeEventEnvelope<RuntimesChangedPayload> {
  if (!isObject(value)) return false;
  if (value.type !== 'runtimeManager.runtimesChanged') return false;
  if (!isObject(value.payload)) return false;

  return Array.isArray(value.payload.runtimes);
}

function isActiveRuntimeChangedEvent(
  value: unknown,
): value is RuntimeEventEnvelope<ActiveRuntimeChangedPayload> {
  if (!isObject(value)) return false;
  if (value.type !== 'runtimeManager.activeRuntimeChanged') return false;
  if (!isObject(value.payload)) return false;

  const payload = value.payload;

  return (
    typeof payload.hostWindowLabel === 'string' &&
    (typeof payload.appId === 'string' || payload.appId === null) &&
    (typeof payload.runtimeId === 'string' || payload.runtimeId === null)
  );
}

function shouldKeepRuntimeEventsMounted() {
  return listeners.size > 0 || activeRuntimeListeners.size > 0;
}

function ensureRuntimeEventsMounted() {
  if (runtimeEventsMounted) {
    return;
  }

  runtimeEventsMounted = true;

  void getCurrentWebview()
    .listen(HOST_RUNTIME_EVENT_NAME, (event) => {
      const data = event.payload;

      if (isRuntimesChangedEvent(data)) {
        notifyRuntimeListeners(data.payload.runtimes);
        return;
      }

      if (isActiveRuntimeChangedEvent(data)) {
        notifyActiveRuntimeListeners(data.payload);
      }
    })
    .then((unlisten) => {
      unlistenRuntimeEvents = unlisten;
    })
    .catch((err) => {
      runtimeEventsMounted = false;
      console.error('Failed to subscribe to runtime manager events:', err);
    });
}

async function refreshRuntimes(): Promise<SageAppRuntimeRecordView[]> {
  if (polling) {
    return cachedRuntimes;
  }

  polling = true;
  try {
    const next = await commands.appsListRuntimes();
    notifyRuntimeListeners(next);
    return next;
  } catch (err) {
    console.error('Failed to refresh app runtimes:', err);
    return cachedRuntimes;
  } finally {
    polling = false;
  }
}

function ensurePolling() {
  ensureRuntimeEventsMounted();

  if (pollTimer != null) {
    return;
  }

  void refreshRuntimes();

  pollTimer = window.setInterval(() => {
    void refreshRuntimes();
  }, 1000);
}

function maybeStopPolling() {
  if (shouldKeepRuntimeEventsMounted()) {
    return;
  }

  if (pollTimer != null) {
    window.clearInterval(pollTimer);
    pollTimer = null;
  }

  if (unlistenRuntimeEvents) {
    unlistenRuntimeEvents();
    unlistenRuntimeEvents = null;
    runtimeEventsMounted = false;
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

export function subscribeActiveRuntime(
  listener: ActiveRuntimeListener,
): () => void {
  activeRuntimeListeners.add(listener);
  ensureRuntimeEventsMounted();

  return () => {
    activeRuntimeListeners.delete(listener);
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
  const args: CreateInstalledRuntimeArgs = {
    appId: app.common.identity.id,
  };

  const created = await commands.appsCreateInlineRuntime(args);
  await refreshRuntimes();
  return created;
}

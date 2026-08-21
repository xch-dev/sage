import {
  createContext,
  type ReactNode,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useRef,
  useState,
} from 'react';
import { listen } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';
import {
  type AppLaunchGateResult,
  commands,
  type ListedSageAppView,
  type SageAppRuntimeRecordView,
  type SandboxStateView,
  type SystemSageAppView,
  type UserSageAppView,
} from '@/bindings';

type UserInstalledEntry = { kind: 'user' } & UserSageAppView;
type SystemInstalledEntry = { kind: 'system' } & SystemSageAppView;
type InstalledEntry = UserInstalledEntry | SystemInstalledEntry;

const SAGE_RUNTIME_EVENT_NAME = 'apps:runtime-event';

export type PendingUpdateStatusView =
  | { kind: 'none' }
  | { kind: 'readyToApply'; manifestHash: string }
  | { kind: 'requiresReview'; manifestHash: string }
  | {
      kind: 'requiresNewerSage';
      manifestHash: string;
      currentVersion: string;
      minimumVersion: string;
    }
  | {
      kind: 'untestedNewerSage';
      manifestHash: string;
      currentVersion: string;
      testedMaxVersion: string;
    };

interface RuntimeManagerRuntimesChangedEvent {
  type: 'runtimeManager.runtimesChanged';
  payload: {
    runtimes: SageAppRuntimeRecordView[];
  };
}

interface ActiveTaskbarRuntimeChangedEvent {
  type: 'runtimeManager.activeTaskbarRuntimeChanged';
  payload: {
    hostWindowLabel: string;
    appId: string | null;
    runtimeId: string | null;
  };
}

interface ListedAppsChangedEvent {
  type: 'appRegistry.listedAppsChanged';
  payload: {
    apps: ListedSageAppView[];
  };
}

interface SandboxStateChangedEvent {
  type: 'sandbox.stateChanged';
  payload: {
    state: SandboxStateView;
  };
}

interface PendingUpdateChangedEvent {
  type: 'appUpdate.pendingUpdateChanged';
  payload: {
    appId: string;
    status: PendingUpdateStatusView;
  };
}

type SageRuntimeEvent =
  | RuntimeManagerRuntimesChangedEvent
  | ActiveTaskbarRuntimeChangedEvent
  | ListedAppsChangedEvent
  | SandboxStateChangedEvent
  | PendingUpdateChangedEvent;

type ActiveTaskbarRuntime = {
  appId: string | null;
  runtimeId: string | null;
} | null;

interface AppsContextValue {
  apps: ListedSageAppView[];
  runtimes: SageAppRuntimeRecordView[];
  taskbarRuntimes: SageAppRuntimeRecordView[];
  loading: boolean;
  error: string | null;
  busyAppIds: Record<string, boolean>;
  pendingUpdates: Record<string, PendingUpdateStatusView>;
  sandboxState: SandboxStateView | null;
  launchGatesByAppId: Record<string, AppLaunchGateResult>;

  getApp: (appId: string) => UserSageAppView | undefined;
  getListedApp: (appId: string) => InstalledEntry | undefined;
  getLaunchGate: (appId: string) => AppLaunchGateResult | null;
  getTaskbarRuntime: (appId: string) => SageAppRuntimeRecordView | null;

  refresh: () => Promise<void>;
  refreshInstalledApps: () => Promise<void>;
  refreshRuntimes: () => Promise<void>;
  refreshLaunchGates: (listed: ListedSageAppView[]) => Promise<void>;
  setBusy: (appId: string, busy: boolean) => void;

  uninstallApp: (appId: string) => Promise<void>;
  clearAppStorage: (appId: string) => Promise<void>;
  activeTaskbarRuntime: ActiveTaskbarRuntime;
}

const AppsContext = createContext<AppsContextValue | null>(null);

function formatError(err: unknown): string {
  if (err instanceof Error) return err.message;
  if (typeof err === 'string') return err;

  try {
    return JSON.stringify(err, null, 2);
  } catch {
    return String(err);
  }
}

function isInstalledEntry(entry: ListedSageAppView): entry is InstalledEntry {
  return entry.kind === 'user' || entry.kind === 'system';
}

function installedAppId(app: InstalledEntry): string {
  return app.common.identity.id;
}

function runtimeAppId(runtime: SageAppRuntimeRecordView): string {
  return runtime.app.common.identity.id;
}

function isUserListedApp(
  entry: ListedSageAppView,
): entry is { kind: 'user' } & UserSageAppView {
  return entry.kind === 'user';
}

function isTaskbarRuntime(runtime: SageAppRuntimeRecordView): boolean {
  return runtime.presentation.kind === 'Taskbar';
}

export function AppsProvider({ children }: { children: ReactNode }) {
  const [apps, setApps] = useState<ListedSageAppView[]>([]);
  const appsRef = useRef<ListedSageAppView[]>([]);

  const [runtimes, setRuntimes] = useState<SageAppRuntimeRecordView[]>([]);
  const [taskbarRuntimes, setTaskbarRuntimes] = useState<
    SageAppRuntimeRecordView[]
  >([]);
  const [activeTaskbarRuntime, setActiveTaskbarRuntime] =
    useState<ActiveTaskbarRuntime>(null);

  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [busyAppIds, setBusyAppIds] = useState<Record<string, boolean>>({});

  const [pendingUpdates, setPendingUpdates] = useState<
    Record<string, PendingUpdateStatusView>
  >({});

  const [sandboxState, setSandboxState] = useState<SandboxStateView | null>(
    null,
  );

  const [launchGatesByAppId, setLaunchGatesByAppId] = useState<
    Record<string, AppLaunchGateResult>
  >({});

  useEffect(() => {
    appsRef.current = apps;
  }, [apps]);

  const refreshRuntimes = useCallback(async () => {
    try {
      const next = await commands.appsListRuntimes();
      setRuntimes(next);
      setTaskbarRuntimes(next.filter(isTaskbarRuntime));
    } catch (err) {
      console.error('Failed to refresh runtimes:', err);
    }
  }, []);

  const refreshLaunchGates = useCallback(
    async (listed: ListedSageAppView[]) => {
      const installed = listed.filter(isInstalledEntry);

      const results = await Promise.allSettled(
        installed.map(async (app) => {
          const appId = installedAppId(app);
          const gate = await commands.appsGetAppLaunchGate(appId);
          return [appId, gate] as const;
        }),
      );

      const next: Record<string, AppLaunchGateResult> = {};

      for (const result of results) {
        if (result.status === 'fulfilled') {
          const [appId, gate] = result.value;
          next[appId] = gate;
        } else {
          console.error('Failed to refresh launch gate:', result.reason);
        }
      }

      setLaunchGatesByAppId(next);
    },
    [],
  );

  const refreshInstalledApps = useCallback(async () => {
    setLoading(true);
    setError(null);

    try {
      const listed = await commands.appsListInstalledApps();

      setApps(listed);
      setLoading(false);

      void (async () => {
        try {
          const sandbox = await commands.appsGetSandboxState();
          setSandboxState(sandbox);
        } catch (err) {
          console.error('Failed to refresh sandbox state:', err);
        }
      })();

      void refreshLaunchGates(listed);
    } catch (err) {
      setError(formatError(err));
      setLoading(false);
    }
  }, [refreshLaunchGates]);

  useEffect(() => {
    void refreshInstalledApps();
    void refreshRuntimes();
  }, [refreshInstalledApps, refreshRuntimes]);

  useEffect(() => {
    let isCancelled = false;
    let unsubscribe: (() => void) | undefined;

    const setup = async () => {
      try {
        unsubscribe = await listen<SageRuntimeEvent>(
          SAGE_RUNTIME_EVENT_NAME,
          (event) => {
            if (isCancelled) return;

            const runtimeEvent = event.payload;

            switch (runtimeEvent.type) {
              case 'runtimeManager.runtimesChanged':
                setRuntimes(runtimeEvent.payload.runtimes);
                setTaskbarRuntimes(
                  runtimeEvent.payload.runtimes.filter(isTaskbarRuntime),
                );
                break;

              case 'runtimeManager.activeTaskbarRuntimeChanged':
                if (
                  runtimeEvent.payload.hostWindowLabel !==
                  getCurrentWindow().label
                ) {
                  break;
                }

                setActiveTaskbarRuntime({
                  appId: runtimeEvent.payload.appId,
                  runtimeId: runtimeEvent.payload.runtimeId,
                });
                break;

              case 'appRegistry.listedAppsChanged':
                setApps(runtimeEvent.payload.apps);
                setLoading(false);
                setError(null);
                void refreshLaunchGates(runtimeEvent.payload.apps);
                break;

              case 'sandbox.stateChanged':
                setSandboxState(runtimeEvent.payload.state);
                void refreshLaunchGates(appsRef.current);
                break;

              case 'appUpdate.pendingUpdateChanged':
                setPendingUpdates((prev) => ({
                  ...prev,
                  [runtimeEvent.payload.appId]: runtimeEvent.payload.status,
                }));
                break;
            }
          },
        );
      } catch (err) {
        if (!isCancelled) {
          console.error('Failed to subscribe to runtime events:', err);
        }
      }
    };

    void setup();

    return () => {
      isCancelled = true;
      unsubscribe?.();
    };
  }, [refreshLaunchGates]);

  const currentSandboxRunId = sandboxState?.currentRun?.runId ?? null;

  useEffect(() => {
    if (!currentSandboxRunId) return;

    let isCancelled = false;

    const refreshSandboxState = async () => {
      try {
        const next = await commands.appsGetSandboxState();
        if (!isCancelled) {
          setSandboxState(next);
          void refreshLaunchGates(appsRef.current);
        }
      } catch (err) {
        if (!isCancelled) {
          console.error('Failed to refresh sandbox state:', err);
        }
      }
    };

    void refreshSandboxState();

    const intervalId = window.setInterval(() => {
      void refreshSandboxState();
    }, 1000);

    return () => {
      isCancelled = true;
      window.clearInterval(intervalId);
    };
  }, [currentSandboxRunId, refreshLaunchGates]);

  const refresh = refreshInstalledApps;

  const getListedApp = useCallback(
    (appId: string): InstalledEntry | undefined => {
      return apps.find(
        (item): item is InstalledEntry =>
          isInstalledEntry(item) && item.common.identity.id === appId,
      );
    },
    [apps],
  );

  const getLaunchGate = useCallback(
    (appId: string): AppLaunchGateResult | null =>
      launchGatesByAppId[appId] ?? null,
    [launchGatesByAppId],
  );

  const getTaskbarRuntime = useCallback(
    (appId: string) => {
      return (
        taskbarRuntimes.find((runtime) => runtimeAppId(runtime) === appId) ??
        null
      );
    },
    [taskbarRuntimes],
  );

  const getApp = useCallback(
    (appId: string): UserSageAppView | undefined => {
      return apps.find(
        (item): item is { kind: 'user' } & UserSageAppView =>
          isUserListedApp(item) && item.common.identity.id === appId,
      );
    },
    [apps],
  );

  const setBusy = useCallback((appId: string, busy: boolean) => {
    setBusyAppIds((prev) => ({ ...prev, [appId]: busy }));
  }, []);

  const uninstallApp = useCallback(
    async (appId: string) => {
      setBusy(appId, true);

      try {
        await commands.appsUninstallApp(appId);

        setPendingUpdates((prev) =>
          Object.fromEntries(
            Object.entries(prev).filter(([key]) => key !== appId),
          ),
        );

        setLaunchGatesByAppId((prev) =>
          Object.fromEntries(
            Object.entries(prev).filter(([key]) => key !== appId),
          ),
        );
      } finally {
        setBusy(appId, false);
      }
    },
    [setBusy],
  );

  const clearAppStorage = useCallback(
    async (appId: string) => {
      await commands.appsClearRuntimeBrowsingData(appId);
      await refreshInstalledApps();
      await refreshRuntimes();
    },
    [refreshInstalledApps, refreshRuntimes],
  );

  const value = useMemo<AppsContextValue>(
    () => ({
      apps,
      runtimes,
      taskbarRuntimes,
      loading,
      error,
      busyAppIds,
      pendingUpdates,
      sandboxState,
      launchGatesByAppId,

      getApp,
      getListedApp,
      getLaunchGate,
      getTaskbarRuntime,

      refresh,
      refreshInstalledApps,
      refreshRuntimes,
      refreshLaunchGates,
      setBusy,

      uninstallApp,
      clearAppStorage,
      activeTaskbarRuntime,
    }),
    [
      apps,
      runtimes,
      taskbarRuntimes,
      loading,
      error,
      busyAppIds,
      pendingUpdates,
      sandboxState,
      launchGatesByAppId,
      getApp,
      getListedApp,
      getLaunchGate,
      getTaskbarRuntime,
      refresh,
      refreshInstalledApps,
      refreshRuntimes,
      refreshLaunchGates,
      setBusy,
      uninstallApp,
      clearAppStorage,
      activeTaskbarRuntime,
    ],
  );

  return <AppsContext.Provider value={value}>{children}</AppsContext.Provider>;
}

export function useApps() {
  const value = useContext(AppsContext);

  if (!value) {
    throw new Error('useApps must be used within AppsProvider');
  }

  return value;
}

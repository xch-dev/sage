import { createContext, type ReactNode, useCallback, useContext, useEffect, useMemo, useState, } from 'react';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';
import {
  type AppLaunchGateResult,
  commands,
  type ListedSageAppView,
  type SageAppRuntimeRecordView,
  type SageAppUrlPreview,
  SageAppView,
  SageGrantedPermissionsInput,
  type SandboxStateView,
  type SystemSageAppView,
  type UserSageAppView,
} from '@/bindings';

interface PerformAppUpdateOptions {
  restartIfRunning?: boolean;
  visibleAfterRestart?: boolean;
}

type UserInstalledEntry = { kind: 'user' } & UserSageAppView;
type SystemInstalledEntry = { kind: 'system' } & SystemSageAppView;
type InstalledEntry = UserInstalledEntry | SystemInstalledEntry;

const SAGE_RUNTIME_EVENT_NAME = 'apps:runtime-event';

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

type SageRuntimeEvent =
  | RuntimeManagerRuntimesChangedEvent
  | ActiveTaskbarRuntimeChangedEvent;

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
  updateAvailability: Record<string, SageAppUrlPreview | null>;
  sandboxState: SandboxStateView | null;
  launchGatesByAppId: Record<string, AppLaunchGateResult>;

  getApp: (appId: string) => UserSageAppView | undefined;
  getListedApp: (appId: string) => InstalledEntry | undefined;
  getLaunchGate: (appId: string) => AppLaunchGateResult | null;
  getTaskbarRuntime: (
    appId: string,
  ) => SageAppRuntimeRecordView | null;

  refresh: () => Promise<void>;
  refreshInstalledApps: () => Promise<void>;
  refreshRuntimes: () => Promise<void>;
  refreshLaunchGates: (listed: ListedSageAppView[]) => Promise<void>;
  setBusy: (appId: string, busy: boolean) => void;
  setUpdateAvailability: (
    updater:
      | Record<string, SageAppUrlPreview | null>
      | ((
          prev: Record<string, SageAppUrlPreview | null>,
        ) => Record<string, SageAppUrlPreview | null>),
  ) => void;

  uninstallApp: (appId: string) => Promise<void>;
  checkForUpdate: (appId: string) => Promise<SageAppUrlPreview | null>;
  performAppUpdate: (
    appId: string,
    grantedPermissions: SageGrantedPermissionsInput,
    options?: PerformAppUpdateOptions,
  ) => Promise<SageAppView>;
  clearAppStorage: (appId: string) => Promise<void>;
  rerunSandboxTests: () => Promise<SandboxStateView>;
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
  const [runtimes, setRuntimes] = useState<SageAppRuntimeRecordView[]>([]);
  const [taskbarRuntimes, setTaskbarRuntimes] = useState<
    SageAppRuntimeRecordView[]
  >([]);
  const [activeTaskbarRuntime, setActiveTaskbarRuntime] =
    useState<ActiveTaskbarRuntime>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [busyAppIds, setBusyAppIds] = useState<Record<string, boolean>>({});
  const [updateAvailability, setUpdateAvailability] = useState<
    Record<string, SageAppUrlPreview | null>
  >({});
  const [sandboxState, setSandboxState] = useState<SandboxStateView | null>(
    null,
  );
  const [launchGatesByAppId, setLaunchGatesByAppId] = useState<
    Record<string, AppLaunchGateResult>
  >({});

  const refreshRuntimes = useCallback(async () => {
    try {
      const next = await commands.appsListRuntimes();
      setRuntimes(next);
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
      const listed = await commands.listInstalledApps();

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
            console.log('Received runtime event:', runtimeEvent);

            switch (runtimeEvent.type) {
              case 'runtimeManager.runtimesChanged':
                setRuntimes(runtimeEvent.payload.runtimes);
                setTaskbarRuntimes(runtimeEvent.payload.runtimes.filter((runtime) => isTaskbarRuntime(runtime)));
                break;

              case 'runtimeManager.activeTaskbarRuntimeChanged':
                if (runtimeEvent.payload.hostWindowLabel !== getCurrentWindow().label) {
                  break;
                }
                setActiveTaskbarRuntime({
                    appId: runtimeEvent.payload.appId,
                    runtimeId: runtimeEvent.payload.runtimeId,
                });
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
  }, []);

  useEffect(() => {
    let isCancelled = false;
    let unsubscribe: UnlistenFn | null = null;

    const setup = async () => {
      try {
        unsubscribe = await listen<SandboxStateView>(
          'apps:sandbox-state-updated',
          (event) => {
            if (isCancelled) return;

            setSandboxState(event.payload);
            void refreshLaunchGates(apps);
          },
        );
      } catch (err) {
        if (!isCancelled) {
          console.error('Failed to subscribe to sandbox state updates:', err);
        }
      }
    };

    void setup();

    return () => {
      isCancelled = true;
      if (unsubscribe) void unsubscribe();
    };
  }, [apps, refreshLaunchGates]);

  const currentSandboxRunId = sandboxState?.currentRun?.runId ?? null;

  useEffect(() => {
    if (!currentSandboxRunId) return;

    let isCancelled = false;

    const refreshSandboxState = async () => {
      try {
        const next = await commands.appsGetSandboxState();
        if (!isCancelled) {
          setSandboxState(next);
          void refreshLaunchGates(apps);
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
  }, [apps, currentSandboxRunId, refreshLaunchGates]);

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
      return taskbarRuntimes.find((runtime) => runtimeAppId(runtime) === appId) ?? null;
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

  const setUpdateAvailabilityState = useCallback(
    (
      updater:
        | Record<string, SageAppUrlPreview | null>
        | ((
            prev: Record<string, SageAppUrlPreview | null>,
          ) => Record<string, SageAppUrlPreview | null>),
    ) => {
      setUpdateAvailability((prev) =>
        typeof updater === 'function' ? updater(prev) : updater,
      );
    },
    [],
  );

  const uninstallApp = useCallback(
    async (appId: string) => {
      setBusy(appId, true);
      try {
        await commands.uninstallApp(appId);

        setUpdateAvailability((prev) =>
          Object.fromEntries(
            Object.entries(prev).filter(([key]) => key !== appId),
          ),
        );

        setLaunchGatesByAppId((prev) =>
          Object.fromEntries(
            Object.entries(prev).filter(([key]) => key !== appId),
          ),
        );

        await refreshInstalledApps();
        await refreshRuntimes();
      } finally {
        setBusy(appId, false);
      }
    },
    [refreshInstalledApps, refreshRuntimes, setBusy],
  );

  const checkForUpdate = useCallback(async (appId: string) => {
    const preview = await commands.checkAppUpdate(appId);

    setUpdateAvailability((prev) => ({ ...prev, [appId]: preview }));

    return preview;
  }, []);

  const performAppUpdate = useCallback(
    async (
      appId: string,
      grantedPermissions: SageGrantedPermissionsInput,
    ) => {
      setBusy(appId, true);
      try {
        await commands.downloadAppUpdate(appId);

        return await commands.applyAppUpdate(appId, grantedPermissions);
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

  const rerunSandboxTests = useCallback(async () => {
    const next = await commands.appsRerunSandboxTests();
    setSandboxState(next);
    await refreshLaunchGates(apps);
    return next;
  }, [apps, refreshLaunchGates]);

  const value = useMemo<AppsContextValue>(
    () => ({
      apps,
      runtimes,
      taskbarRuntimes,
      loading,
      error,
      busyAppIds,
      updateAvailability,
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
      setUpdateAvailability: setUpdateAvailabilityState,

      uninstallApp,
      checkForUpdate,
      performAppUpdate,
      clearAppStorage,
      rerunSandboxTests,
      activeTaskbarRuntime,
    }),
    [
      apps,
      runtimes,
      taskbarRuntimes,
      loading,
      error,
      busyAppIds,
      updateAvailability,
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
      setUpdateAvailabilityState,
      uninstallApp,
      checkForUpdate,
      performAppUpdate,
      clearAppStorage,
      rerunSandboxTests,
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

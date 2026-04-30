import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from 'react';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import {
  type AppLaunchGateResult,
  commands,
  type ListedSageAppView,
  type SageAppUrlPreview,
  SageAppView,
  SageGrantedPermissionsInput,
  type SandboxStateView,
  type SystemSageAppView,
  type UserSageAppView,
} from '@/bindings';
import { useAppPendingApprovals } from '@/hooks/useAppPendingApprovals';
import { useBridgeHost } from '@/hooks/useBridgeHost';
import type { PendingApprovalItem } from '@/hooks/useAppPendingApprovals';

interface PerformAppUpdateOptions {
  restartIfRunning?: boolean;
  visibleAfterRestart?: boolean;
}

type UserInstalledEntry = { kind: 'user' } & UserSageAppView;
type SystemInstalledEntry = { kind: 'system' } & SystemSageAppView;
type InstalledEntry = UserInstalledEntry | SystemInstalledEntry;

interface AppsContextValue {
  apps: ListedSageAppView[];
  loading: boolean;
  error: string | null;
  busyAppIds: Record<string, boolean>;
  updateAvailability: Record<string, SageAppUrlPreview | null>;
  bridgeHostReady: boolean;
  sandboxState: SandboxStateView | null;
  launchGatesByAppId: Record<string, AppLaunchGateResult>;

  currentApproval: PendingApprovalItem | null;
  queuedApprovalCount: number;
  currentApprovalSecondsLeft: number;
  approveCurrentApproval: () => void;
  rejectCurrentApproval: () => void;

  getApp: (appId: string) => UserSageAppView | undefined;
  getListedApp: (appId: string) => InstalledEntry | undefined;
  getLaunchGate: (appId: string) => AppLaunchGateResult | null;

  refresh: () => Promise<void>;
  refreshInstalledApps: () => Promise<void>;
  refreshLaunchGates: (listed?: ListedSageAppView[]) => Promise<void>;
  setBusy: (appId: string, busy: boolean) => void;
  setUpdateAvailability: (
    updater:
      | Record<string, SageAppUrlPreview | null>
      | ((
          prev: Record<string, SageAppUrlPreview | null>,
        ) => Record<string, SageAppUrlPreview | null>),
  ) => void;

  installApp: (
    zipPath: string,
    grantedPermissions: SageGrantedPermissionsInput,
  ) => Promise<UserSageAppView>;
  installUrlApp: (
    appUrl: string,
    grantedPermissions: SageGrantedPermissionsInput,
  ) => Promise<UserSageAppView>;
  uninstallApp: (appId: string) => Promise<void>;
  checkForUpdate: (appId: string) => Promise<SageAppUrlPreview | null>;
  performAppUpdate: (
    appId: string,
    grantedPermissions: SageGrantedPermissionsInput,
    options?: PerformAppUpdateOptions,
  ) => Promise<SageAppView>;
  clearAppStorage: (appId: string) => Promise<void>;
  rerunSandboxTests: () => Promise<SandboxStateView>;
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

function isLaunchGateEntry(
  entry: readonly [string, AppLaunchGateResult] | null,
): entry is readonly [string, AppLaunchGateResult] {
  return entry !== null;
}

function isInstalledEntry(entry: ListedSageAppView): entry is InstalledEntry {
  return entry.kind === 'user' || entry.kind === 'system';
}

function isUserListedApp(
  entry: ListedSageAppView,
): entry is { kind: 'user' } & UserSageAppView {
  return entry.kind === 'user';
}

export function AppsProvider({ children }: { children: ReactNode }) {
  const [apps, setApps] = useState<ListedSageAppView[]>([]);
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

  const {
    currentApproval,
    queuedApprovalCount,
    currentApprovalSecondsLeft,
    requestApproval,
    approveCurrentApproval,
    rejectCurrentApproval,
  } = useAppPendingApprovals();

  const { isReady: bridgeHostReady } = useBridgeHost({ requestApproval });

  const refreshLaunchGates = useCallback(
    async (listed: ListedSageAppView[] = apps) => {
      const entries = await Promise.all(
        listed.filter(isInstalledEntry).map(async (app) => {
          try {
            return [
              app.common.identity.id,
              await commands.appsGetAppLaunchGate(app.common.identity.id),
            ] as const;
          } catch (err) {
            console.error(
              `Failed to refresh launch gate for ${app.common.identity.id}:`,
              err,
            );
            return null;
          }
        }),
      );

      setLaunchGatesByAppId(
        Object.fromEntries(entries.filter(isLaunchGateEntry)),
      );
    },
    [apps],
  );

  const refreshInstalledApps = useCallback(async () => {
    setLoading(true);
    setError(null);

    try {
      const [listed, sandbox] = await Promise.all([
        commands.listInstalledApps(),
        commands.appsGetSandboxState(),
      ]);

      setApps(listed);
      setSandboxState(sandbox);

      const entries = await Promise.all(
        listed.filter(isInstalledEntry).map(async (app) => {
          try {
            return [
              app.common.identity.id,
              await commands.appsGetAppLaunchGate(app.common.identity.id),
            ] as const;
          } catch (err) {
            console.error(
              `Failed to refresh launch gate for ${app.common.identity.id}:`,
              err,
            );
            return null;
          }
        }),
      );

      setLaunchGatesByAppId(
        Object.fromEntries(entries.filter(isLaunchGateEntry)),
      );
    } catch (err) {
      setError(formatError(err));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void refreshInstalledApps();
  }, [refreshInstalledApps]);

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
            void refreshLaunchGates();
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
          void refreshLaunchGates();
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

  const installApp = useCallback(
    async (zipPath: string, grantedPermissions: SageGrantedPermissionsInput) => {
      const installed = await commands.installAppZip(
        zipPath,
        grantedPermissions,
      );
      await refreshInstalledApps();
      return installed;
    },
    [refreshInstalledApps],
  );

  const installUrlApp = useCallback(
    async (appUrl: string, grantedPermissions: SageGrantedPermissionsInput) => {
      const installed = await commands.installAppUrl(
        appUrl,
        grantedPermissions,
      );
      await refreshInstalledApps();
      return installed;
    },
    [refreshInstalledApps],
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
      } finally {
        setBusy(appId, false);
      }
    },
    [refreshInstalledApps, setBusy],
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
      options?: PerformAppUpdateOptions,
    ) => {
      setBusy(appId, true);
      try {
        await commands.downloadAppUpdate(appId);

        const installed = await commands.applyAppUpdate(
          appId,
          grantedPermissions,
        );

        if (options?.restartIfRunning) {
          const { restartAppRuntime } =
            await import('@/lib/apps/restartAppRuntime');

          try {
            await restartAppRuntime(installed, {
              visible: options.visibleAfterRestart ?? true,
            });
          } catch {
            //
          }
        }

        setUpdateAvailability((prev) => ({ ...prev, [appId]: null }));

        await refreshInstalledApps();
        return installed;
      } finally {
        setBusy(appId, false);
      }
    },
    [refreshInstalledApps, setBusy],
  );

  const clearAppStorage = useCallback(
    async (appId: string) => {
      await commands.appsClearRuntimeBrowsingData(appId);
      await refreshInstalledApps();
    },
    [refreshInstalledApps],
  );

  const rerunSandboxTests = useCallback(async () => {
    const next = await commands.appsRerunSandboxTests();
    setSandboxState(next);
    await refreshLaunchGates();
    return next;
  }, [refreshLaunchGates]);

  const value = useMemo<AppsContextValue>(
    () => ({
      apps,
      loading,
      error,
      busyAppIds,
      updateAvailability,
      bridgeHostReady,
      sandboxState,
      launchGatesByAppId,

      currentApproval,
      queuedApprovalCount,
      currentApprovalSecondsLeft,
      approveCurrentApproval,
      rejectCurrentApproval,

      getApp,
      getListedApp,
      getLaunchGate,

      refresh,
      refreshInstalledApps,
      refreshLaunchGates,
      setBusy,
      setUpdateAvailability: setUpdateAvailabilityState,

      installApp,
      installUrlApp,
      uninstallApp,
      checkForUpdate,
      performAppUpdate,
      clearAppStorage,
      rerunSandboxTests,
    }),
    [
      apps,
      loading,
      error,
      busyAppIds,
      updateAvailability,
      bridgeHostReady,
      sandboxState,
      launchGatesByAppId,
      currentApproval,
      queuedApprovalCount,
      currentApprovalSecondsLeft,
      approveCurrentApproval,
      rejectCurrentApproval,
      getApp,
      getListedApp,
      getLaunchGate,
      refresh,
      refreshInstalledApps,
      refreshLaunchGates,
      setBusy,
      setUpdateAvailabilityState,
      installApp,
      installUrlApp,
      uninstallApp,
      checkForUpdate,
      performAppUpdate,
      clearAppStorage,
      rerunSandboxTests,
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

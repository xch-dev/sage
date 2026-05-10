import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { CorruptedAppCard } from '@/components/apps/CorruptedAppCard';
import { AppsLaunchpadContextMenu } from '@/components/apps/AppsLaunchpadContextMenu';
import { Button } from '@/components/ui/button';
import { formatSandboxLaunchDecision } from '@/lib/apps/sandboxPolicy';
import {
  commands,
  ListedSageAppView,
  SystemSageAppView,
  UserSageAppView,
} from '@/bindings.ts';
import { useApps } from '@/contexts/AppsContext.tsx';
import { Plus } from 'lucide-react';
import { AppsPageActionsMenu } from '@/components/apps/AppsPageActionsMenu';
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { AppTile } from '@/components/apps/AppTile';
import { formatAppError } from '@/lib/apps/formatAppError.ts';
import {
  openAppPermissionsReview,
} from '@/lib/apps/openAppUpdate.ts';

type UserInstalledEntry = { kind: 'user' } & UserSageAppView;
type SystemInstalledEntry = { kind: 'system' } & SystemSageAppView;
type InstalledEntry = UserInstalledEntry | SystemInstalledEntry;
type CorruptedEntry = Extract<ListedSageAppView, { kind: 'corrupted' }>;

type AppContextMenuState = {
  app: InstalledEntry;
  x: number;
  y: number;
} | null;

function isInstalledEntry(entry: ListedSageAppView): entry is InstalledEntry {
  return entry.kind === 'user' || entry.kind === 'system';
}

function isUserInstalledEntry(
  entry: InstalledEntry,
): entry is UserInstalledEntry {
  return entry.kind === 'user';
}

function isCorruptedEntry(entry: ListedSageAppView): entry is CorruptedEntry {
  return entry.kind === 'corrupted';
}

function clampContextMenuPosition(args: {
  x: number;
  y: number;
  containerWidth: number;
  containerHeight: number;
}) {
  const menuWidth = 260;
  const menuHeight = 260;
  const padding = 8;

  return {
    x: Math.max(
      padding,
      Math.min(args.x, args.containerWidth - menuWidth - padding),
    ),
    y: Math.max(
      padding,
      Math.min(args.y, args.containerHeight - menuHeight - padding),
    ),
  };
}

function formatErrorMessage(err: unknown): string {
  if (err instanceof Error) {
    return err.message;
  }

  if (typeof err === 'string') {
    return err;
  }

  try {
    return JSON.stringify(err, null, 2);
  } catch {
    return String(err);
  }
}

async function openApp(appId: string) {
  return await commands.appsCreateInlineRuntime({
    appId,
  });
}

export function AppsLaunchpad() {
  const [contextMenu, setContextMenu] = useState<AppContextMenuState>(null);
  const pageRef = useRef<HTMLDivElement | null>(null);

  const [updateCheckStateByAppId, setUpdateCheckStateByAppId] = useState<
    Record<string, 'idle' | 'up_to_date'>
  >({});

  const [clearingDataByAppId, setClearingDataByAppId] = useState<
    Record<string, boolean>
  >({});

  const [clearDataErrorByAppId, setClearDataErrorByAppId] = useState<
    Record<string, string | null>
  >({});

  const {
    apps,
    runtimes,
    loading,
    error,
    refresh,
    uninstallApp,
    clearAppStorage,
    pendingUpdates,
    busyAppIds,
    getLaunchGate,
  } = useApps();

  const runningAppIds = useMemo(() => {
    return new Set(runtimes.map((runtime) => runtime.app.common.identity.id));
  }, [runtimes]);

  const installedApps = useMemo(
    () =>
      apps.filter((entry): entry is InstalledEntry => isInstalledEntry(entry)),
    [apps],
  );

  const corruptedApps = useMemo(() => apps.filter(isCorruptedEntry), [apps]);

  const contextMenuAppId = contextMenu?.app.common.identity.id ?? null;

  const contextMenuPendingUpdate = contextMenuAppId
    ? (pendingUpdates[contextMenuAppId] ?? { kind: 'none' as const })
    : { kind: 'none' as const };

  const contextMenuBusy = contextMenuAppId
    ? (busyAppIds[contextMenuAppId] ?? false)
    : false;

  const contextMenuCheckState =
    contextMenuAppId
      ? (updateCheckStateByAppId[contextMenuAppId] ?? 'idle')
      : 'idle';

  const contextMenuAppIsRunning = contextMenuAppId
    ? runningAppIds.has(contextMenuAppId)
    : false;

  const contextMenuClearDataBusy = contextMenuAppId
    ? (clearingDataByAppId[contextMenuAppId] ?? false)
    : false;

  const contextMenuClearDataError = contextMenuAppId
    ? (clearDataErrorByAppId[contextMenuAppId] ?? null)
    : null;

  const contextMenuHasUpdate = contextMenuPendingUpdate.kind !== 'none';
  const contextMenuUpdateIsInstallable =
    contextMenuPendingUpdate.kind === 'readyToApply';

  const closeContextMenu = useCallback(() => {
    setContextMenu((prevContextMenu) => {
      if (prevContextMenu) {
        setUpdateCheckStateByAppId((prev) => {
          if (prev[prevContextMenu.app.common.identity.id] !== 'up_to_date') {
            return prev;
          }

          return {
            ...prev,
            [prevContextMenu.app.common.identity.id]: 'idle',
          };
        });
      }

      return null;
    });
  }, []);

  async function handleCheckForUpdate(appId: string) {
    setUpdateCheckStateByAppId((prev) => ({
      ...prev,
      [appId]: 'idle',
    }));

    setClearDataErrorByAppId((prev) => ({
      ...prev,
      [appId]: null,
    }));

    try {
      const preview = await commands.checkAppUpdate(appId);

      if (!preview) {
        setUpdateCheckStateByAppId((prev) => ({
          ...prev,
          [appId]: 'up_to_date',
        }));
      }
    } catch (err) {
      const message = formatAppError(err);

      console.error('checkAppUpdate failed:', err);

      setClearDataErrorByAppId((prev) => ({
        ...prev,
        [appId]: `Update check failed: ${message}`,
      }));
    }
  }

  async function handleApplyUpdate(appId: string) {
    setClearDataErrorByAppId((prev) => ({
      ...prev,
      [appId]: null,
    }));

    try {
      await commands.applyAppUpdate(appId);
    } catch (err) {
      const message = formatAppError(err);

      console.error('applyAppUpdate failed:', err);

      setClearDataErrorByAppId((prev) => ({
        ...prev,
        [appId]: `Update failed: ${message}`,
      }));
    }
  }

  const handleClearData = useCallback(
    async (app: InstalledEntry) => {
      const appId = app.common.identity.id;

      setClearingDataByAppId((prev) => ({
        ...prev,
        [appId]: true,
      }));

      setClearDataErrorByAppId((prev) => ({
        ...prev,
        [appId]: null,
      }));

      try {
        await clearAppStorage(appId);
        await refresh();
      } catch (err) {
        setClearDataErrorByAppId((prev) => ({
          ...prev,
          [appId]: formatErrorMessage(err),
        }));
      } finally {
        setClearingDataByAppId((prev) =>
          Object.fromEntries(
            Object.entries(prev).filter(([key]) => key !== appId),
          ),
        );
      }
    },
    [clearAppStorage, refresh],
  );

  useEffect(() => {
    if (!contextMenu || contextMenuCheckState !== 'up_to_date') {
      return;
    }

    const timeoutId = window.setTimeout(() => {
      setUpdateCheckStateByAppId((prev) => {
        if (prev[contextMenu.app.common.identity.id] !== 'up_to_date') {
          return prev;
        }

        return {
          ...prev,
          [contextMenu.app.common.identity.id]: 'idle',
        };
      });
    }, 3000);

    return () => {
      window.clearTimeout(timeoutId);
    };
  }, [contextMenu, contextMenuCheckState]);

  useEffect(() => {
    if (!contextMenu) {
      return;
    }

    const handleClose = () => {
      if (clearingDataByAppId[contextMenu.app.common.identity.id]) {
        return;
      }

      closeContextMenu();
    };

    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        if (clearingDataByAppId[contextMenu.app.common.identity.id]) {
          return;
        }

        closeContextMenu();
      }
    };

    window.addEventListener('click', handleClose);
    window.addEventListener('resize', handleClose);
    window.addEventListener('scroll', handleClose, true);
    window.addEventListener('keydown', handleKeyDown);

    return () => {
      window.removeEventListener('click', handleClose);
      window.removeEventListener('resize', handleClose);
      window.removeEventListener('scroll', handleClose, true);
      window.removeEventListener('keydown', handleKeyDown);
    };
  }, [contextMenu, clearingDataByAppId, closeContextMenu]);

  if (loading) {
    return (
      <div className='mx-auto w-full max-w-6xl p-4 md:p-6'>
        <Alert>
          <AlertTitle>Loading apps...</AlertTitle>
          <AlertDescription>Please wait.</AlertDescription>
        </Alert>
      </div>
    );
  }

  return (
    <div
      ref={pageRef}
      className='relative flex h-full min-h-0 flex-col overflow-hidden'
    >
      <div className='mx-auto flex w-full max-w-7xl shrink-0 items-center justify-between gap-4 p-4 md:p-6'>
        <div>
          <h1 className='text-2xl font-semibold tracking-tight'>Apps</h1>
          <p className='text-sm text-muted-foreground'>
            Launch and manage installed Sage apps.
          </p>
        </div>

        <div className='flex items-center gap-2'>
          <Button
            variant='outline'
            onClick={() => {
              void commands.appsStartSystemApp({
                kind: 'appInstall',
                source: { kind: 'selectSource' },
              });
            }}
          >
            <Plus className='mr-2 h-4 w-4' />
            Install App
          </Button>

          <AppsPageActionsMenu
            onOpenSandboxTests={() => {
              void commands.appsStartSystemApp({
                kind: 'sandboxTests',
              });
            }}
            onClose={() => {
              //
            }}
          />
        </div>
      </div>

      <div className='mx-auto w-full max-w-7xl flex-1 min-h-0 overflow-auto px-4 pb-4 md:px-6 md:pb-6'>
        {error ? (
          <Alert className='mb-6'>
            <AlertTitle>Apps error</AlertTitle>
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        ) : null}

        {installedApps.length === 0 ? (
          <Alert className='mb-6'>
            <AlertTitle>No apps installed</AlertTitle>
            <AlertDescription>
              Install a Sage app package to get started.
            </AlertDescription>
          </Alert>
        ) : null}

        {installedApps.length > 0 ? (
          <div className='grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6'>
            {installedApps.map((app) => (
              <AppTile
                key={app.common.identity.id}
                app={app}
                launchDecision={
                  app.kind === 'system'
                    ? {
                        allowed: true,
                        title: 'System app',
                        description: 'System apps are managed by Sage.',
                      }
                    : formatSandboxLaunchDecision(
                        getLaunchGate(app.common.identity.id),
                      )
                }
                onOpen={() => {
                  void openApp(app.common.identity.id);
                }}
                onContextMenu={(event) => {
                  event.preventDefault();

                  const pageEl = pageRef.current;
                  if (!pageEl) {
                    return;
                  }

                  const pageRect = pageEl.getBoundingClientRect();

                  const localX = event.clientX - pageRect.left;
                  const localY = event.clientY - pageRect.top;

                  const position = clampContextMenuPosition({
                    x: localX,
                    y: localY,
                    containerWidth: pageRect.width,
                    containerHeight: pageRect.height,
                  });

                  setClearDataErrorByAppId((prev) => ({
                    ...prev,
                    [app.common.identity.id]: null,
                  }));

                  setContextMenu({
                    app,
                    x: position.x,
                    y: position.y,
                  });
                }}
              />
            ))}
          </div>
        ) : null}

        {corruptedApps.length > 0 ? (
          <div className='mt-8 space-y-4'>
            <div>
              <h2 className='text-lg font-semibold tracking-tight'>
                Corrupted apps
              </h2>
              <p className='text-sm text-muted-foreground'>
                These app installations could not be loaded correctly.
              </p>
            </div>

            <div className='space-y-4'>
              {corruptedApps.map((entry) => (
                <CorruptedAppCard
                  key={entry.id}
                  app={entry}
                  onRemove={() => uninstallApp(entry.id)}
                />
              ))}
            </div>
          </div>
        ) : null}
      </div>

      <AppsLaunchpadContextMenu
        open={!!contextMenu}
        x={contextMenu?.x ?? 0}
        y={contextMenu?.y ?? 0}
        busy={contextMenuBusy}
        hasUpdate={contextMenuHasUpdate}
        updateIsInstallable={contextMenuUpdateIsInstallable}
        isRunning={contextMenuAppIsRunning}
        updateCheckState={contextMenuCheckState}
        clearDataBusy={contextMenuClearDataBusy}
        clearDataError={contextMenuClearDataError}
        onClose={closeContextMenu}
        onOpen={() => {
          if (!contextMenu) {
            return;
          }

          setUpdateCheckStateByAppId((prev) => ({
            ...prev,
            [contextMenu.app.common.identity.id]: 'idle',
          }));

          void openApp(contextMenu.app.common.identity.id);
          closeContextMenu();
        }}
        onCheckForUpdate={() => {
          if (!contextMenu || !isUserInstalledEntry(contextMenu.app)) {
            return;
          }

          void handleCheckForUpdate(contextMenu.app.common.identity.id);
        }}
        onUpdate={() => {
          if (!contextMenu || !isUserInstalledEntry(contextMenu.app)) {
            return;
          }

          const appId = contextMenu.app.common.identity.id;

          closeContextMenu();

          void handleApplyUpdate(appId);
        }}
        onChangePermissions={() => {
          if (!contextMenu || !isUserInstalledEntry(contextMenu.app)) {
            return;
          }

          const appId = contextMenu.app.common.identity.id;

          void openAppPermissionsReview(appId);
          closeContextMenu();
        }}
        onClearData={() => {
          if (!contextMenu) {
            return;
          }

          void handleClearData(contextMenu.app);
        }}
        onUninstall={() => {
          if (!contextMenu || !isUserInstalledEntry(contextMenu.app)) {
            return;
          }

          setUpdateCheckStateByAppId((prev) => ({
            ...prev,
            [contextMenu.app.common.identity.id]: 'idle',
          }));

          void uninstallApp(contextMenu.app.common.identity.id).finally(() => {
            closeContextMenu();
          });
        }}
      />
    </div>
  );
}

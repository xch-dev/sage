import { useEffect, useMemo, useRef, useState } from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';

import { commands, type UserSageAppView } from '@/bindings';
import { useApps } from '@/contexts/AppsContext.tsx';

import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Button } from '@/components/ui/button';

import {
  AppTaskBar,
  type AppTaskBarTab,
} from '@/components/apps/AppTaskBar.tsx';

import { AppHost } from '@/components/apps/AppHost';
import { AppsLaunchpad } from '@/components/apps/AppsLaunchpad';
import { SystemAppModalLayer } from '@/components/apps/SystemAppModalLayer';

export function Apps() {
  const [workspaceActive, setWorkspaceActive] = useState(false);

  useEffect(() => {
    let cancelled = false;

    void commands
      .appsEnterWorkspace()
      .then(() => {
        if (!cancelled) {
          setWorkspaceActive(true);
        }
      })
      .catch((err) => {
        console.error('Failed to activate apps workspace:', err);
      });

    return () => {
      cancelled = true;

      setWorkspaceActive(false);

      void commands.appsLeaveWorkspace().catch((err) => {
        console.error('Failed to deactivate apps workspace:', err);
      });
    };
  }, []);

  const {
    runtimes,
    getApp,
    getListedApp,
    pendingUpdates,
    busyAppIds,
    activeTaskbarRuntime,
  } = useApps();

  const runtimesRef = useRef(runtimes);

  useEffect(() => {
    runtimesRef.current = runtimes;
  }, [runtimes]);

  useEffect(() => {
    if (!import.meta.env.DEV) {
      return;
    }

    let cleanup: (() => void) | null = null;

    void import('@/dev/system-apps/setupDevSystemAppsReload').then(
      ({ setupDevSystemAppsReload }) => {
        cleanup = setupDevSystemAppsReload(() => runtimesRef.current);
      },
    );

    return () => {
      cleanup?.();
    };
  }, []);

  const activeRuntimeAppId = activeTaskbarRuntime?.appId ?? null;

  const activeRuntimeExists = activeRuntimeAppId
    ? runtimes.some(
        (runtime) => runtime.app.common.identity.id === activeRuntimeAppId,
      )
    : false;

  const activeApp: UserSageAppView | null =
    activeRuntimeAppId && activeRuntimeExists
      ? (getApp(activeRuntimeAppId) ?? null)
      : null;

  const activeAppId = activeApp?.common.identity.id ?? null;

  const activePendingUpdate = activeAppId
    ? (pendingUpdates[activeAppId] ?? { kind: 'none' as const })
    : { kind: 'none' as const };

  const activeBusy = activeAppId ? (busyAppIds[activeAppId] ?? false) : false;

  const activeManifest = activeApp?.common.activeSnapshot.manifest;

  const hasDonation = !!activeManifest?.donation?.address;

  const tabs = useMemo<AppTaskBarTab[]>(() => {
    const out: AppTaskBarTab[] = [];

    const orderedRuntimes = [...runtimes].sort(
      (a, b) => a.taskbarOrder - b.taskbarOrder,
    );

    for (const runtime of orderedRuntimes) {
      const installedApp = getListedApp(runtime.app.common.identity.id);

      if (!installedApp) {
        continue;
      }

      if (
        installedApp.kind === 'system' &&
        runtime.presentation.kind !== 'Taskbar'
      ) {
        continue;
      }

      out.push({
        app: installedApp,
        isActive:
          runtime.app.common.identity.id === activeTaskbarRuntime?.appId,
      });
    }

    return out;
  }, [runtimes, getListedApp, activeTaskbarRuntime?.appId]);

  async function handleApplyActiveUpdate() {
    if (!activeAppId) {
      return;
    }

    try {
      await commands.appsApplyAppUpdate(activeAppId);
    } catch (err) {
      console.error('Failed to apply app update:', err);
    }
  }

  if (!workspaceActive) {
    return null;
  }

  return (
    <div className='relative flex h-full min-h-0 w-full flex-col overflow-hidden'>
      <AppTaskBar
        tabs={tabs}
        activeAppId={activeTaskbarRuntime?.appId ?? null}
        onOpenApps={() => {
          void commands.appsClearActiveTaskbarRuntime({
            windowLabel: getCurrentWindow().label,
          });
        }}
        onSelectApp={(tab) => {
          void commands.appsFocusTaskbarRuntime({
            appId: tab.app.common.identity.id,
          });
        }}
        onCloseApp={(tab) => {
          void commands.appsKillTaskbarRuntime({
            appId: tab.app.common.identity.id,
          });
        }}
        onReorderTabs={(appIds) => {
          void commands
            .appsReorderTaskbarRuntimes({
              windowLabel: getCurrentWindow().label,
              appIds,
            })
            .catch((err) => {
              console.error('Failed to reorder taskbar runtimes', err);
            });
        }}
        activeAppHasDonation={hasDonation}
        onOpenDonation={() => {
          if (!activeApp) {
            return;
          }

          void commands.appsStartSystemApp({
            kind: 'donation',
            appId: activeApp.common.identity.id,
          });
        }}
      />

      {activeApp?.source.kind === 'url' &&
      activePendingUpdate.kind !== 'none' ? (
        <Alert className='shrink-0 rounded-none border-x-0 border-t-0'>
          <AlertTitle>
            {activePendingUpdate.kind === 'requiresReview'
              ? 'Update needs review'
              : 'Update ready'}
          </AlertTitle>

          <AlertDescription className='flex items-center justify-between gap-4'>
            <span>
              {activePendingUpdate.kind === 'requiresReview'
                ? `An update is available for ${activeApp.common.activeSnapshot.manifest.name} and needs review before it can be applied.`
                : `An update is ready to apply for ${activeApp.common.activeSnapshot.manifest.name}.`}
            </span>

            <Button
              variant='outline'
              disabled={activeBusy}
              onClick={() => {
                if (!activeAppId) {
                  return;
                }
                void handleApplyActiveUpdate();
              }}
            >
              {activePendingUpdate.kind === 'requiresReview'
                ? 'Review update'
                : 'Apply update'}
            </Button>
          </AlertDescription>
        </Alert>
      ) : null}

      <div className='relative flex-1 min-h-0 overflow-hidden'>
        {activeTaskbarRuntime?.appId ? <AppHost /> : <AppsLaunchpad />}

        <SystemAppModalLayer />
      </div>
    </div>
  );
}

import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Button } from '@/components/ui/button';
import {
  AppTaskBar,
  type AppTaskBarTab,
} from '@/components/apps/AppTaskBar.tsx';
import { useApps } from '@/contexts/AppsContext.tsx';
import { routeForApp } from '../lib/apps/route';
import { useEffect, useMemo, useRef, useState } from 'react';
import { Outlet, useNavigate, useParams } from 'react-router-dom';
import {
  commands,
  type SageAppUrlPreview,
  type UserSageAppView,
} from '@/bindings';
import { SystemAppModalLayer } from '@/components/apps/SystemAppModalLayer';
import { openAppUpdateReview } from '@/lib/apps/openAppUpdate.ts';
import { getCurrentWindow } from '@tauri-apps/api/window';

export function AppsWorkspace() {
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

  const { appId } = useParams();
  const navigate = useNavigate();

  const {
    runtimes,
    getApp,
    getListedApp,
    updateAvailability,
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

  useEffect(() => {
    const activeAppId = activeTaskbarRuntime?.appId ?? null;

    if (!activeAppId) {
      if (appId) {
        navigate('/apps', { replace: true });
      }
      return;
    }

    if (activeAppId === appId) {
      return;
    }

    const app = getListedApp(activeAppId);
    const route = app ? routeForApp(app) : null;

    if (route) {
      navigate(route, { replace: true });
    }
  }, [activeTaskbarRuntime?.appId, appId, getListedApp, navigate]);

  const [tabOrder, setTabOrder] = useState<string[]>([]);

  useEffect(() => {
    setTabOrder((prev) => {
      const runtimeIds = runtimes
        .filter((runtime) => {
          const installedApp = getListedApp(runtime.app.common.identity.id);
          if (!installedApp) {
            return false;
          }

          if (installedApp.kind === 'user') {
            return true;
          }

          return runtime.presentation.kind === 'Taskbar';
        })
        .map((runtime) => runtime.app.common.identity.id);

      const kept = prev.filter((runtimeAppId) =>
        runtimeIds.includes(runtimeAppId),
      );

      const added = runtimeIds.filter(
        (runtimeAppId) => !kept.includes(runtimeAppId),
      );

      return [...kept, ...added];
    });
  }, [runtimes, getListedApp]);

  const activeApp: UserSageAppView | null = appId
    ? (getApp(appId) ?? null)
    : null;

  const activeUpdatePreview: SageAppUrlPreview | null = activeApp
    ? (updateAvailability[activeApp.common.identity.id] ?? null)
    : null;

  const activeBusy = activeApp
    ? (busyAppIds[activeApp.common.identity.id] ?? false)
    : false;

  const activeManifest = activeApp?.common.activeSnapshot.manifest;
  const hasDonation = !!activeManifest?.donation?.address;

  const tabs = useMemo<AppTaskBarTab[]>(() => {
    const runtimeByAppId = new Map(
      runtimes.map(
        (runtime) => [runtime.app.common.identity.id, runtime] as const,
      ),
    );

    const out: AppTaskBarTab[] = [];

    for (const runtimeAppId of tabOrder) {
      const runtime = runtimeByAppId.get(runtimeAppId);
      if (!runtime) continue;

      const installedApp = getListedApp(runtime.app.common.identity.id);
      if (!installedApp) continue;

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
  }, [runtimes, tabOrder, getListedApp, activeTaskbarRuntime?.appId]);

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
        onReorderTabs={setTabOrder}
        activeAppHasDonation={hasDonation}
        onOpenDonation={() => {
          if (!activeApp) return;
          void commands.appsStartSystemApp({
            kind: 'donation',
            appId: activeApp.common.identity.id,
          });
        }}
      />

      {activeApp?.source.kind === 'url' && activeUpdatePreview ? (
        <Alert className='shrink-0 rounded-none border-x-0 border-t-0'>
          <AlertTitle>New version available</AlertTitle>
          <AlertDescription className='flex items-center justify-between gap-4'>
            <span>
              {activeUpdatePreview.manifest.kind === 'full'
                ? `Version ${activeUpdatePreview.manifest.manifest.version} is available for ${activeApp.common.activeSnapshot.manifest.name}.`
                : `An update is available for ${activeApp.common.activeSnapshot.manifest.name}, but it cannot be installed by this Sage version.`}
            </span>

            <Button
              variant='outline'
              disabled={activeBusy}
              onClick={() => {
                if (!activeApp) return;
                void openAppUpdateReview(activeApp.common.identity.id);
              }}
            >
              Review update
            </Button>
          </AlertDescription>
        </Alert>
      ) : null}

      <div className='relative flex-1 min-h-0 overflow-hidden'>
        <Outlet />
        <SystemAppModalLayer />
      </div>
    </div>
  );
}

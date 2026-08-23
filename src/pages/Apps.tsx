import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { platform } from '@tauri-apps/plugin-os';
import { snapshotWebview } from 'tauri-plugin-sage';

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
import { AppTabOverview } from '@/components/apps/AppTabOverview';
import { MobileAppTray } from '@/components/apps/MobileAppTray';
import { SystemAppModalLayer } from '@/components/apps/SystemAppModalLayer';

export function Apps() {
  const isIos = platform() === 'ios';
  const [workspaceActive, setWorkspaceActive] = useState(false);
  const [tabOverviewOpen, setTabOverviewOpen] = useState(false);
  const [tabPreviews, setTabPreviews] = useState<Record<string, string>>({});
  const mobileTransitionQueueRef = useRef<Promise<void>>(Promise.resolve());

  const enqueueMobileTransition = useCallback(
    (transition: () => Promise<void>) => {
      const result = mobileTransitionQueueRef.current.then(
        transition,
        transition,
      );

      mobileTransitionQueueRef.current = result.catch(() => undefined);

      return result;
    },
    [],
  );

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

  const activeRuntimeAppId = activeTaskbarRuntime?.appId ?? null;

  const activeRuntime = activeRuntimeAppId
    ? (runtimes.find(
        (runtime) =>
          runtime.app.common.identity.id === activeRuntimeAppId &&
          runtime.presentation.kind === 'Taskbar',
      ) ?? null)
    : null;

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
    const runtimeByAppId = new Map(
      runtimes.map(
        (runtime) => [runtime.app.common.identity.id, runtime] as const,
      ),
    );

    const out: AppTaskBarTab[] = [];

    for (const runtimeAppId of tabOrder) {
      const runtime = runtimeByAppId.get(runtimeAppId);

      if (!runtime) {
        continue;
      }

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
  }, [runtimes, tabOrder, getListedApp, activeTaskbarRuntime?.appId]);

  useEffect(() => {
    const appIds = new Set(tabs.map((tab) => tab.app.common.identity.id));

    setTabPreviews((previous) => {
      const entries = Object.entries(previous).filter(([appId]) =>
        appIds.has(appId),
      );

      return entries.length === Object.keys(previous).length
        ? previous
        : Object.fromEntries(entries);
    });
  }, [tabs]);

  useEffect(() => {
    if (activeRuntimeAppId) {
      setTabOverviewOpen(false);
    }
  }, [activeRuntimeAppId]);

  async function captureActivePreview() {
    if (!isIos || !activeRuntime) {
      return;
    }

    try {
      const dataUrl = await snapshotWebview(activeRuntime.webviewLabel);
      setTabPreviews((previous) => ({
        ...previous,
        [activeRuntime.app.common.identity.id]: dataUrl,
      }));
    } catch (err) {
      console.warn('Failed to capture app tab preview:', err);
    }
  }

  function showLaunchpad() {
    return enqueueMobileTransition(async () => {
      await captureActivePreview();
      await commands.appsClearActiveTaskbarRuntime({
        windowLabel: getCurrentWindow().label,
      });
      setTabOverviewOpen(false);
    });
  }

  function showTabOverview() {
    return enqueueMobileTransition(async () => {
      await captureActivePreview();
      await commands.appsClearActiveTaskbarRuntime({
        windowLabel: getCurrentWindow().label,
      });
      setTabOverviewOpen(true);
    });
  }

  function prepareNavigation() {
    return enqueueMobileTransition(async () => {
      await commands.appsSetModalRuntimesSuspended({ suspended: true });

      try {
        await commands.appsClearActiveTaskbarRuntime({
          windowLabel: getCurrentWindow().label,
        });
        setTabOverviewOpen(false);
      } catch (err) {
        await commands.appsSetModalRuntimesSuspended({ suspended: false });
        throw err;
      }
    });
  }

  function finishNavigation() {
    return enqueueMobileTransition(async () => {
      await commands.appsSetModalRuntimesSuspended({ suspended: false });
    });
  }

  function selectMobileTab(tab: AppTaskBarTab) {
    void enqueueMobileTransition(async () => {
      await commands.appsFocusTaskbarRuntime({
        appId: tab.app.common.identity.id,
      });
      setTabOverviewOpen(false);
    }).catch((err) => {
      console.error('Failed to select mobile app tab:', err);
    });
  }

  function closeMobileTab(tab: AppTaskBarTab) {
    const appId = tab.app.common.identity.id;

    void enqueueMobileTransition(async () => {
      await commands.appsKillTaskbarRuntime({ appId });

      setTabPreviews((previous) =>
        Object.fromEntries(
          Object.entries(previous).filter(
            ([previewAppId]) => previewAppId !== appId,
          ),
        ),
      );
    }).catch((err) => {
      console.error('Failed to close mobile app tab:', err);
    });
  }

  function openActiveDonation() {
    if (!activeApp) {
      return;
    }

    void enqueueMobileTransition(async () => {
      await commands.appsStartSystemApp({
        kind: 'donation',
        appId: activeApp.common.identity.id,
      });
    }).catch((err) => {
      console.error('Failed to open developer support:', err);
    });
  }

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
      {!isIos ? (
        <AppTaskBar
          tabs={tabs}
          activeAppId={activeTaskbarRuntime?.appId ?? null}
          onOpenApps={() => {
            void showLaunchpad().catch((err) => {
              console.error('Failed to show apps launchpad:', err);
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
          onOpenDonation={openActiveDonation}
        />
      ) : null}

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
        {isIos && tabOverviewOpen ? (
          <AppTabOverview
            tabs={tabs}
            previews={tabPreviews}
            onSelectApp={selectMobileTab}
            onCloseApp={closeMobileTab}
          />
        ) : activeTaskbarRuntime?.appId ? (
          <AppHost />
        ) : (
          <AppsLaunchpad />
        )}

        <SystemAppModalLayer />
      </div>

      {isIos ? (
        <MobileAppTray
          openAppCount={tabs.length}
          overviewOpen={tabOverviewOpen}
          activeAppHasDonation={hasDonation}
          onPrepareNavigation={prepareNavigation}
          onFinishNavigation={finishNavigation}
          onOpenApps={() => {
            void showLaunchpad().catch((err) => {
              console.error('Failed to show apps launchpad:', err);
            });
          }}
          onOpenOverview={() => {
            void showTabOverview().catch((err) => {
              console.error('Failed to show mobile app tabs:', err);
            });
          }}
          onOpenDonation={openActiveDonation}
        />
      ) : null}
    </div>
  );
}

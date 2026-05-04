import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Button } from '@/components/ui/button';
import { AppApprovalStrip } from '@/components/apps/AppApprovalStrip.tsx';
import {
  AppTaskBar,
  type AppTaskBarTab,
} from '@/components/apps/AppTaskBar.tsx';
import { useApps } from '@/contexts/AppsContext.tsx';
import { focusRuntime, killRuntime } from '@/lib/apps/runtimeRegistry';
import { routeForApp } from '@/lib/apps/types';
import { useEffect, useMemo, useRef, useState } from 'react';
import { Outlet, useNavigate, useParams } from 'react-router-dom';
import {
  commands,
  type SageAppUrlPreview,
  type UserSageAppView,
} from '@/bindings';
import { AppDonationStrip } from '@/components/apps/AppDonationStrip.tsx';
import { SystemAppModalLayer } from '@/components/apps/SystemAppModalLayer';
import { openAppUpdateReview } from '@/lib/apps/openAppUpdate.ts';

export function AppsWorkspace() {
  const { appId } = useParams();
  const navigate = useNavigate();

  const {
    runtimes,
    getApp,
    getListedApp,
    updateAvailability,
    busyAppIds,
    currentApproval,
    queuedApprovalCount,
    currentApprovalSecondsLeft,
    approveCurrentApproval,
    rejectCurrentApproval,
    activeRuntimeByHostWindowLabel,
    currentHostWindowLabel,
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

  const activeRuntime =
    activeRuntimeByHostWindowLabel[currentHostWindowLabel] ?? null;

  useEffect(() => {
    const activeAppId = activeRuntime?.appId;

    if (!activeAppId || activeAppId === appId) {
      return;
    }

    const app = getListedApp(activeAppId);
    if (!app) {
      return;
    }

    const route = routeForApp(app);
    if (!route) {
      return;
    }

    navigate(route, { replace: true });
  }, [activeRuntime?.appId, appId, getListedApp, navigate]);

  const [approvalExpanded, setApprovalExpanded] = useState(false);
  const [tabOrder, setTabOrder] = useState<string[]>([]);
  const [donationOpen, setDonationOpen] = useState(false);

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

  useEffect(() => {
    setApprovalExpanded(false);
  }, [currentApproval?.id]);

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
        isActive: runtime.app.common.identity.id === activeRuntime?.appId,
      });
    }

    return out;
  }, [runtimes, tabOrder, getListedApp, activeRuntime?.appId]);

  return (
    <div className='relative flex h-full min-h-0 w-full flex-col overflow-hidden'>
      <AppTaskBar
        tabs={tabs}
        activeAppId={activeRuntime?.appId ?? appId ?? null}
        onOpenApps={() => {
          navigate('/apps');
        }}
        onSelectApp={(tab) => {
          void focusRuntime(tab.app.common.identity.id);
        }}
        onCloseApp={(tab) => {
          const tabAppId = tab.app.common.identity.id;

          void killRuntime(tabAppId).then(() => {
            if (tabAppId === appId) {
              navigate('/apps');
            }
          });
        }}
        onReorderTabs={setTabOrder}
        activeAppHasDonation={hasDonation}
        onOpenDonation={() => setDonationOpen((v) => !v)}
      />

      {activeApp &&
      currentApproval &&
      currentApproval.request.app.common.identity.id ===
        activeApp.common.identity.id ? (
        <AppApprovalStrip
          approval={{
            approvalId: currentApproval.id,
            approval: currentApproval.request,
          }}
          expanded={approvalExpanded}
          queuedApprovalCount={queuedApprovalCount}
          secondsLeft={currentApprovalSecondsLeft}
          onToggleExpanded={() => {
            setApprovalExpanded((prev) => !prev);
          }}
          onApprove={approveCurrentApproval}
          onReject={rejectCurrentApproval}
        />
      ) : null}

      {donationOpen && activeApp && activeManifest?.donation ? (
        <AppDonationStrip
          appName={activeApp.common.activeSnapshot.manifest.name}
          authorName={activeManifest.author?.name}
          authorAvatarSrc={
            activeManifest.author?.avatar
              ? `sage-app://${activeApp.common.identity.originId}/${activeManifest.author.avatar}`
              : null
          }
          donationAddress={activeManifest.donation.address}
          onSend={(amountMojos) => {
            if (!activeManifest.donation) {
              return;
            }

            void commands.sendXch({
              address: activeManifest.donation.address,
              amount: amountMojos,
              fee: '0',
              memos: [],
              auto_submit: false,
            });
          }}
        />
      ) : null}

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

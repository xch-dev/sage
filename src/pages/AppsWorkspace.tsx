import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Button } from '@/components/ui/button';
import { AppApprovalStrip } from '@/components/apps/AppApprovalStrip.tsx';
import {
  AppTaskBar,
  type AppTaskBarTab,
} from '@/components/apps/AppTaskBar.tsx';
import { useApps } from '@/contexts/AppsContext.tsx';
import { useAppRuntimes } from '@/hooks/useAppRuntimes';
import {
  focusRuntime,
  killRuntime,
  subscribeActiveRuntime,
} from '@/lib/apps/runtimeRegistry';
import { formatAppError } from '@/lib/apps/formatAppError';
import { routeForApp } from '@/lib/apps/types';
import { useCallback, useEffect, useMemo, useState } from 'react';
import { Outlet, useNavigate, useParams } from 'react-router-dom';
import {
  commands,
  type SageAppUrlPreview,
  type SageGrantedPermissionsView,
  type UserSageAppView,
} from '@/bindings';
import { AppUpdateDialog } from '@/components/apps/AppUpdateDialog.tsx';
import { getAppUpdatePermissionsDelta } from '@/lib/apps/updatePermissionsDelta.ts';
import { AppDonationStrip } from '@/components/apps/AppDonationStrip.tsx';
import { SystemAppModalLayer } from '@/components/apps/SystemAppModalLayer';

export function AppsWorkspace() {
  const { appId } = useParams();
  const navigate = useNavigate();
  const runtimes = useAppRuntimes();

  const {
    getApp,
    getListedApp,
    updateAvailability,
    busyAppIds,
    performAppUpdate,
    currentApproval,
    queuedApprovalCount,
    currentApprovalSecondsLeft,
    approveCurrentApproval,
    rejectCurrentApproval,
  } = useApps();

  const [approvalExpanded, setApprovalExpanded] = useState(false);
  const [applyingUpdate, setApplyingUpdate] = useState(false);
  const [tabOrder, setTabOrder] = useState<string[]>([]);
  const [updateDialogOpen, setUpdateDialogOpen] = useState(false);
  const [updateDialogError, setUpdateDialogError] = useState<string | null>(
    null,
  );
  const [donationOpen, setDonationOpen] = useState(false);

  useEffect(() => {
    return subscribeActiveRuntime((event) => {
      if (!event.appId || event.appId === appId) {
        return;
      }

      const app = getListedApp(event.appId);
      if (!app) {
        return;
      }

      const route = routeForApp(app);
      if (!route) {
        return;
      }

      navigate(route, { replace: true });
    });
  }, [appId, getListedApp, navigate]);

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

          return installedApp.presentation === 'Taskbar';
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
        installedApp.presentation !== 'Taskbar'
      ) {
        continue;
      }

      out.push({
        app: installedApp,
        isActive: runtime.app.common.identity.id === appId,
      });
    }

    return out;
  }, [runtimes, tabOrder, getListedApp, appId]);

  const handleConfirmUpdate = useCallback(
    async (nextGrantedPermissions: SageGrantedPermissionsView) => {
      if (!activeApp) {
        return;
      }

      try {
        setApplyingUpdate(true);
        setUpdateDialogError(null);

        await performAppUpdate(
          activeApp.common.identity.id,
          nextGrantedPermissions,
          {
            restartIfRunning: true,
            visibleAfterRestart: true,
          },
        );

        setUpdateDialogOpen(false);
      } catch (err) {
        setUpdateDialogError(formatAppError(err));
      } finally {
        setApplyingUpdate(false);
      }
    },
    [activeApp, performAppUpdate],
  );

  const handleReviewOrApplyUpdate = useCallback(async () => {
    if (!activeApp || !activeUpdatePreview) {
      return;
    }

    if (activeUpdatePreview.manifest.kind === 'partial') {
      setUpdateDialogError(null);
      setUpdateDialogOpen(true);
      return;
    }

    const delta = getAppUpdatePermissionsDelta(activeApp, activeUpdatePreview);

    if (!delta.requiresUserReview) {
      setUpdateDialogOpen(false);
      setUpdateDialogError(null);
      await handleConfirmUpdate(delta.nextGrantedPermissions);
      return;
    }

    setUpdateDialogError(null);
    setUpdateDialogOpen(true);
  }, [activeApp, activeUpdatePreview, handleConfirmUpdate]);

  return (
    <div className='relative flex h-full min-h-0 w-full flex-col overflow-hidden'>
      <AppTaskBar
        tabs={tabs}
        activeAppId={appId ?? null}
        onOpenApps={() => {
          navigate('/apps');
        }}
        onSelectApp={(tab) => {
          const nextRoute = routeForApp(tab.app);
          if (!nextRoute) {
            return;
          }

          void focusRuntime(tab.app.common.identity.id).then(() => {
            navigate(nextRoute);
          });
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
              disabled={activeBusy || applyingUpdate}
              onClick={() => {
                void handleReviewOrApplyUpdate();
              }}
            >
              {applyingUpdate ? 'Updating...' : 'Review update'}
            </Button>
          </AlertDescription>
        </Alert>
      ) : null}

      <div className='flex-1 min-h-0 overflow-hidden'>
        <Outlet />
      </div>

      <SystemAppModalLayer />

      <AppUpdateDialog
        open={updateDialogOpen}
        app={activeApp}
        preview={activeUpdatePreview}
        submitting={applyingUpdate}
        error={updateDialogError}
        onCancel={() => {
          if (!applyingUpdate) {
            setUpdateDialogOpen(false);
            setUpdateDialogError(null);
          }
        }}
        onConfirm={(nextGranted) => {
          void handleConfirmUpdate(nextGranted);
        }}
      />
    </div>
  );
}

import React, { useEffect, useMemo, useState } from 'react';
import { AppModalShell } from '@sage-app/ui';
import {
  useSageSystemClient,
  type PendingBridgeApprovalView,
  type SageAppRuntimeRecordView,
} from '@sage-system-app/sdk';
import { BadgeCheck } from 'lucide-react';
import { AppApprovalBody } from './approval/AppApprovalBody';

function appNameFromRuntime(runtime: SageAppRuntimeRecordView | null): string {
  return runtime?.app.common.activeSnapshot.manifest.name ?? 'Unknown app';
}

function appIdFromRuntime(
  runtime: SageAppRuntimeRecordView | null,
): string | null {
  return runtime?.app.common.identity.id ?? null;
}

function titleForApproval(approval: PendingBridgeApprovalView) {
  switch (approval.approval.kind) {
    case 'sendXch':
      return 'Approve XCH transaction';
    case 'getSecretKey':
      return 'Approve secret key access';
    case 'capabilityGrant':
      return 'Approve permission grant';
    case 'networkWhitelistGrant':
      return 'Approve network access';
  }
}

function queueText(count: number) {
  if (count <= 0) return null;
  return `${count} more approval${count === 1 ? '' : 's'} pending`;
}

function MetaPill({ children }: { children: React.ReactNode }) {
  return (
    <span className='rounded-full border px-2 py-0.5 text-[10px] uppercase tracking-wide text-muted-foreground'>
      {children}
    </span>
  );
}

export function App() {
  const sage = useSageSystemClient();

  const [approvals, setApprovals] = useState<PendingBridgeApprovalView[]>([]);
  const [activeAppId, setActiveAppId] = useState<string | null>(null);
  const [activeAppName, setActiveAppName] = useState('Unknown app');
  const [expanded, setExpanded] = useState(false);
  const [working, setWorking] = useState(false);
  const [loaded, setLoaded] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function refreshActiveRuntime() {
    const active = await sage.runtimeManager.getActiveRuntime();

    setActiveAppId(appIdFromRuntime(active));
    setActiveAppName(appNameFromRuntime(active));

    return active;
  }

  useEffect(() => {
    async function refreshInitialState() {
      try {
        const [pending, active] = await Promise.all([
          sage.bridgeApprovals.listPending(),
          sage.runtimeManager.getActiveRuntime(),
        ]);

        setApprovals(pending);
        setActiveAppId(appIdFromRuntime(active));
        setActiveAppName(appNameFromRuntime(active));
      } catch (err) {
        console.error('[approval] refreshInitialState failed', err);
        setError(err instanceof Error ? err.message : String(err));
      } finally {
        setLoaded(true);
      }
    }

    void refreshInitialState();
  }, [sage]);

  useEffect(() => {
    const offApprovals = sage.bridgeApprovals.onChanged((event) => {
      setApprovals(event.approvals);
    });

    const offActiveRuntime = sage.runtimeManager.onActiveRuntimeChanged(() => {
      void refreshActiveRuntime().catch((err) => {
        console.error('[approval] failed to refresh active runtime', err);
      });
    });

    return () => {
      offApprovals();
      offActiveRuntime();
    };
  }, [sage]);

  const activeApproval = useMemo(() => {
    if (!activeAppId) return null;
    return approvals.find((item) => item.appId === activeAppId) ?? null;
  }, [approvals, activeAppId]);

  const queuedApprovalCount = useMemo(() => {
    if (!activeAppId || !activeApproval) return 0;

    return Math.max(
      0,
      approvals.filter((item) => item.appId === activeAppId).length - 1,
    );
  }, [approvals, activeAppId, activeApproval]);

  useEffect(() => {
    setExpanded(false);
    setError(null);
  }, [activeApproval?.approvalId]);

  useEffect(() => {
    if (!loaded || activeApproval) return;

    void sage.runtimeManager.hideSelf().catch((err) => {
      console.error('[approval] failed to hide self', err);
    });
  }, [sage, loaded, activeApproval]);

  async function resolve(approved: boolean) {
    if (!activeApproval || working) return;

    setWorking(true);
    setError(null);

    try {
      await sage.bridgeApprovals.resolve({
        approvalId: activeApproval.approvalId,
        approved,
        reason: approved ? null : 'User denied the request',
      });

      setApprovals(await sage.bridgeApprovals.listPending());
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setWorking(false);
    }
  }

  if (!loaded || !activeApproval) {
    return null;
  }

  const moreText = queueText(queuedApprovalCount);

  return (
    <AppModalShell
      title={titleForApproval(activeApproval)}
      appName='Bridge Approval'
      appIcon={null}
      footer={
        <div className='flex items-center justify-between gap-3'>
          <button
            type='button'
            disabled={working}
            onClick={() => setExpanded((prev) => !prev)}
            className='rounded-md px-3 py-2 text-sm text-muted-foreground hover:bg-muted hover:text-foreground disabled:opacity-50'
          >
            {expanded ? 'Less' : 'More'}
          </button>

          <div className='flex items-center gap-2'>
            <button
              type='button'
              disabled={working}
              onClick={() => void resolve(false)}
              className='rounded-md border border-border px-3 py-2 text-sm hover:bg-muted disabled:opacity-50'
            >
              Reject
            </button>

            <button
              type='button'
              disabled={working}
              onClick={() => void resolve(true)}
              className='rounded-md bg-primary px-3 py-2 text-sm font-medium text-primary-foreground hover:opacity-90 disabled:opacity-50'
            >
              Approve
            </button>
          </div>
        </div>
      }
    >
      <div className='space-y-4'>
        <div>
          <div className='flex flex-wrap items-center gap-2'>
            <div className='text-sm font-semibold'>Approval required</div>
            {moreText ? <MetaPill>{moreText}</MetaPill> : null}
          </div>

          <div className='mt-1 flex items-center gap-2 text-xs text-muted-foreground'>
            <BadgeCheck className='h-3.5 w-3.5' />
            <span>{activeAppName}</span>
            <span>·</span>
            <span className='font-mono'>{activeApproval.appId}</span>
          </div>
        </div>

        <AppApprovalBody
          approval={activeApproval.approval}
          appName={activeAppName}
          expanded={expanded}
        />

        {error ? (
          <div className='rounded-lg border border-destructive/40 bg-destructive/10 p-2 text-sm text-destructive'>
            {error}
          </div>
        ) : null}
      </div>
    </AppModalShell>
  );
}

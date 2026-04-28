import { useMemo } from 'react';
import { Button } from '@/components/ui/button.tsx';
import { BadgeCheck } from 'lucide-react';
import { AppApprovalBody } from '@/components/apps/approval/AppApprovalBody.tsx';
import { RustBridgeApprovalEvent } from '@/bindings.ts';

interface Props {
  approval: RustBridgeApprovalEvent;
  expanded: boolean;
  queuedApprovalCount: number;
  secondsLeft: number;
  onToggleExpanded: () => void;
  onApprove: () => void;
  onReject: () => void;
}

function formatCountdown(secondsLeft: number) {
  if (secondsLeft <= 0) {
    return 'Expires now';
  }

  return `Expires in ${secondsLeft}s`;
}

function MetaPill({ children }: { children: React.ReactNode }) {
  return (
    <span className='rounded-full border px-2 py-0.5 text-[10px] uppercase tracking-wide text-muted-foreground'>
      {children}
    </span>
  );
}

export function AppApprovalStrip({
  approval,
  expanded,
  queuedApprovalCount,
  secondsLeft,
  onToggleExpanded,
  onApprove,
  onReject,
}: Props) {
  const queueText = useMemo(() => {
    if (queuedApprovalCount <= 0) {
      return null;
    }

    return `${queuedApprovalCount} more approval${
      queuedApprovalCount === 1 ? '' : 's'
    } pending`;
  }, [queuedApprovalCount]);

  if (!approval) {
    return null;
  }

  return (
    <div className='shrink-0 border-b bg-muted/30'>
      <div className='px-4 py-3'>
        <div className='rounded-2xl border bg-background/80 p-4 shadow-sm'>
          <div className='mb-4 flex items-start justify-between gap-4'>
            <div className='min-w-0'>
              <div className='flex flex-wrap items-center gap-2'>
                <div className='text-sm font-semibold'>Approval required</div>
                <MetaPill>{formatCountdown(secondsLeft)}</MetaPill>
                {queueText ? <MetaPill>{queueText}</MetaPill> : null}
              </div>

              <div className='mt-1 flex items-center gap-2 text-xs text-muted-foreground'>
                <BadgeCheck className='h-3.5 w-3.5' />
                <span>{approval.approval.app.common.activeSnapshot.manifest.name}</span>
                <span>·</span>
                <span>{approval.approval.requestId}</span>
              </div>
            </div>

            <div className='flex shrink-0 items-center gap-2'>
              <Button variant='ghost' size='sm' onClick={onToggleExpanded}>
                {expanded ? 'Less' : 'More'}
              </Button>
              <Button variant='outline' size='sm' onClick={onReject}>
                Reject
              </Button>
              <Button size='sm' onClick={onApprove}>
                Approve
              </Button>
            </div>
          </div>

          <AppApprovalBody
            approval={approval}
            expanded={expanded}
          />
        </div>
      </div>
    </div>
  );
}

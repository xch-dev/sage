import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Button } from '@/components/ui/button';
import { PendingApprovalItem } from '@/hooks/useAppPendingApprovals.ts';

interface Props {
  currentApproval: PendingApprovalItem | null;
  queuedApprovalCount: number;
  currentApprovalSecondsLeft: number;
  onApprove: () => void;
  onReject: () => void;
}

function renderApprovalDetails(currentApproval: PendingApprovalItem) {
  const req = currentApproval.request;

  switch (req.kind) {
    case 'sendXch':
      return (
        <div className='rounded-md border p-3 text-xs font-mono whitespace-pre-wrap break-all'>
          {JSON.stringify(req.summary, null, 2)}
        </div>
      );

    case 'capabilityGrant':
      return (
        <div className='rounded-md border p-3 text-xs'>
          Grant capability: <span className='font-mono'>{req.capability}</span>
        </div>
      );

    case 'networkWhitelistGrant':
      return (
        <div className='rounded-md border p-3 text-xs'>
          Grant network access:{' '}
          <span className='font-mono'>
            {req.entry.scheme}://{req.entry.host}
          </span>
        </div>
      );
  }
}

export function AppApprovalBanner({
  currentApproval,
  queuedApprovalCount,
  currentApprovalSecondsLeft,
  onApprove,
  onReject,
}: Props) {
  if (!currentApproval) {
    return null;
  }

  const req = currentApproval.request;

  return (
    <Alert>
      <AlertTitle>Approval required</AlertTitle>
      <AlertDescription className='space-y-3'>
        <div className='text-sm'>
          App <strong>{req.app.common.activeSnapshot.manifest.name}</strong> wants to perform{' '}
          <span className='font-mono'>{req.kind}</span>.
        </div>

        <div className='text-xs text-muted-foreground'>
          Expires in {currentApprovalSecondsLeft}s
          {queuedApprovalCount > 0
            ? ` · ${queuedApprovalCount} more approval${
                queuedApprovalCount === 1 ? '' : 's'
              } pending`
            : ''}
        </div>

        {renderApprovalDetails(currentApproval)}

        <div className='flex gap-2'>
          <Button variant='outline' onClick={onReject}>
            Reject
          </Button>

          <Button onClick={onApprove}>Approve</Button>
        </div>
      </AlertDescription>
    </Alert>
  );
}

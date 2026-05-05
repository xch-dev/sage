import type { RustBridgeApprovalRequest } from '@sage-system-app/sdk';
import { Globe } from 'lucide-react';
import { ApprovalDetailRow, ApprovalMetaPill } from './shared';

interface Props {
  approval: Extract<
    RustBridgeApprovalRequest,
    { kind: 'networkWhitelistGrant' }
  >;
  appName: string;
  expanded: boolean;
}

export function NetworkWhitelistGrantApprovalCard({ approval, appName }: Props) {
  const target = `${approval.entry.scheme}://${approval.entry.host}`;

  return (
    <div className='space-y-3'>
      <div className='flex items-start gap-3'>
        <div className='rounded-xl border bg-background p-2 text-muted-foreground'>
          <Globe className='h-4 w-4' />
        </div>

        <div className='min-w-0 flex-1'>
          <div className='flex flex-wrap items-center gap-2'>
            <div className='text-sm font-medium'>Grant network access</div>
            <ApprovalMetaPill>Network</ApprovalMetaPill>
          </div>

          <div className='mt-1 text-xs text-muted-foreground'>
            {appName} wants to contact an additional network
            target.
          </div>
        </div>
      </div>

      <div className='space-y-2 rounded-xl border bg-background/70 p-3'>
        <ApprovalDetailRow label='Target' value={target} mono breakAll />
      </div>
    </div>
  );
}

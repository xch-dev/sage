import type { RustBridgeApprovalRequest } from '@sage-system-app/sdk';
import { KeyRound, ShieldAlert } from 'lucide-react';
import { ApprovalDetailRow, ApprovalMetaPill } from './shared';

interface Props {
  approval: Extract<RustBridgeApprovalRequest, { kind: 'capabilityGrant' }>;
  appName: string,
  expanded: boolean;
}

export function CapabilityGrantApprovalCard({ approval, appName, expanded }: Props) {
  const label = approval.definition.label;
  const description = approval.definition.description;

  return (
    <div className='space-y-3'>
      <div className='flex items-start gap-3'>
        <div className='rounded-xl border bg-background p-2 text-muted-foreground'>
          <KeyRound className='h-4 w-4' />
        </div>

        <div className='min-w-0 flex-1'>
          <div className='flex flex-wrap items-center gap-2'>
            <div className='text-sm font-medium'>Grant permission</div>
            <ApprovalMetaPill>Capability</ApprovalMetaPill>
          </div>

          <div className='mt-1 text-xs text-muted-foreground'>
            {appName} wants access to an additional capability.
          </div>
        </div>
      </div>

      <div className='space-y-2 rounded-xl border bg-background/70 p-3'>
        <ApprovalDetailRow label='Permission' value={label} />
        {description ? (
          <ApprovalDetailRow label='Why' value={description} />
        ) : null}
      </div>

      {expanded ? (
        <div className='flex items-start gap-2 rounded-lg border border-muted px-3 py-2 text-xs text-muted-foreground'>
          <ShieldAlert className='mt-0.5 h-4 w-4 shrink-0' />
          <div className='break-all'>
            Internal key:{' '}
            <span className='font-mono'>{approval.capability}</span>
          </div>
        </div>
      ) : null}
    </div>
  );
}

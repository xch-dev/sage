import { ExternalLink } from 'lucide-react';
import type { RustBridgeApprovalRequest } from 'sage-system-app-sdk';
import { ApprovalDetailRow, ApprovalMetaPill } from './shared';

interface Props {
  approval: Extract<RustBridgeApprovalRequest, { kind: 'openExternalUrl' }>;
  appName: string;
}

export function OpenExternalUrlApprovalCard({ approval, appName }: Props) {
  let host = approval.url;

  try {
    host = new URL(approval.url).host;
  } catch {
    // The host validates this URL before creating the approval request.
  }

  return (
    <div className='space-y-3'>
      <div className='flex items-start gap-3'>
        <div className='rounded-xl border bg-background p-2 text-muted-foreground'>
          <ExternalLink className='h-4 w-4' />
        </div>

        <div className='min-w-0 flex-1'>
          <div className='flex flex-wrap items-center gap-2'>
            <div className='text-sm font-medium'>Open external link</div>
            <ApprovalMetaPill>Browser</ApprovalMetaPill>
          </div>

          <div className='mt-1 text-xs text-muted-foreground'>
            {appName} wants to open this link in your default browser. The
            destination can observe the request.
          </div>
        </div>
      </div>

      <div className='space-y-2 rounded-xl border bg-background/70 p-3'>
        <ApprovalDetailRow label='Destination' value={host} breakAll />
        <ApprovalDetailRow
          label='Full URL'
          value={approval.url}
          mono
          breakAll
        />
      </div>
    </div>
  );
}

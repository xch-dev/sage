import { Wallet } from 'lucide-react';
import {
  ApprovalDetailRow,
  ApprovalMetaPill,
} from '@/components/apps/approval/shared.tsx';
import type { RustBridgeApprovalRequest } from '@/bindings';

interface Props {
  approval: Extract<RustBridgeApprovalRequest, { kind: 'getSecretKey' }>;
  expanded: boolean;
}

export function GetSecretKeyApprovalCard({ approval }: Props) {
  const fingerprint = approval.fingerprint;

  return (
    <div className='space-y-3'>
      <div className='flex items-start gap-3'>
        <div className='rounded-xl border bg-background p-2 text-muted-foreground'>
          <Wallet className='h-4 w-4' />
        </div>

        <div className='min-w-0 flex-1'>
          <div className='flex flex-wrap items-center gap-2'>
            <div className='text-sm font-medium'>Get Secret Key</div>
            <ApprovalMetaPill>Wallet</ApprovalMetaPill>
          </div>

          <div className='mt-1 text-xs text-muted-foreground'>
            {approval.app.common.activeSnapshot.manifest.name} wants to get your secret key.
          </div>
        </div>
      </div>

      <div className='space-y-2 rounded-xl border bg-background/70 p-3'>
        <ApprovalDetailRow label='Fingerprint' value={fingerprint} />
      </div>
    </div>
  );
}

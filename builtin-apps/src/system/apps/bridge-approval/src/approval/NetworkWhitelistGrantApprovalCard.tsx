import type { RustBridgeApprovalRequest } from '@sage-system-app/sdk';

interface Props {
  approval: Extract<
    RustBridgeApprovalRequest,
    { kind: 'networkWhitelistGrant' }
  >;
  appName: string;
  expanded: boolean;
}

export function NetworkWhitelistGrantApprovalCard({ approval }: Props) {
  const target = `${approval.entry.scheme}://${approval.entry.host}`;

  return (
    <div className='rounded-xl border bg-background/70 p-3'>
      <div className='text-sm font-medium'>Network access</div>

      <div className='mt-1 break-all font-mono text-xs text-muted-foreground'>
        {target}
      </div>
    </div>
  );
}

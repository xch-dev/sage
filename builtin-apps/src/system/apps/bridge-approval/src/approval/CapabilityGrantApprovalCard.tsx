import type { RustBridgeApprovalRequest } from 'sage-system-app-sdk';
import { ApprovalDetailRow } from './shared';

interface Props {
  approval: Extract<RustBridgeApprovalRequest, { kind: 'capabilityGrant' }>;
  appName: string;
  expanded: boolean;
}

export function CapabilityGrantApprovalCard({
  approval,
  appName,
  expanded,
}: Props) {
  const label = approval.definition.label;
  const description = approval.definition.description;

  return (
    <div className='rounded-xl border bg-background/70 p-3'>
      <div className='text-sm font-medium'>{label}</div>

      <div className='mt-1 text-xs text-muted-foreground'>
        {description ?? `${appName} wants access to this permission.`}
      </div>

      {expanded ? (
        <div className='mt-3 border-t pt-3'>
          <ApprovalDetailRow
            label='Internal key'
            value={approval.capability}
            mono
            breakAll
          />
        </div>
      ) : null}
    </div>
  );
}

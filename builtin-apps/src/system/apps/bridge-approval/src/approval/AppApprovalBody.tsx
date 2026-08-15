import type { RustBridgeApprovalRequest } from '@sage-system-app/sdk';
import { CapabilityGrantApprovalCard } from './CapabilityGrantApprovalCard';
import { GetSecretKeyApprovalCard } from './GetSecretKeyApprovalCard';
import { NetworkWhitelistGrantApprovalCard } from './NetworkWhitelistGrantApprovalCard';
import { SendXchApprovalCard } from './SendXchApprovalCard';

interface Props {
  approval: RustBridgeApprovalRequest;
  appName: string;
  expanded: boolean;
}

export function AppApprovalBody({ approval, appName, expanded }: Props) {
  switch (approval.kind) {
    case 'getSecretKey':
      return (
        <GetSecretKeyApprovalCard
          approval={approval}
          appName={appName}
          expanded={expanded}
        />
      );

    case 'sendXch':
      return (
        <SendXchApprovalCard
          approval={approval}
          appName={appName}
          expanded={expanded}
        />
      );

    case 'capabilityGrant':
      return (
        <CapabilityGrantApprovalCard
          approval={approval}
          appName={appName}
          expanded={expanded}
        />
      );

    case 'networkWhitelistGrant':
      return (
        <NetworkWhitelistGrantApprovalCard
          approval={approval}
          appName={appName}
          expanded={expanded}
        />
      );
  }
}

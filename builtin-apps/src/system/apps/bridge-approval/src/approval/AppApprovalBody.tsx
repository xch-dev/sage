import type { RustBridgeApprovalRequest } from 'sage-system-app-sdk';
import { CapabilityGrantApprovalCard } from './CapabilityGrantApprovalCard';
import { GetSecretKeyApprovalCard } from './GetSecretKeyApprovalCard';
import { NetworkWhitelistGrantApprovalCard } from './NetworkWhitelistGrantApprovalCard';
import { SendXchApprovalCard } from './SendXchApprovalCard';
import { SignCoinSpendsApprovalCard } from './SignCoinSpendsApprovalCard';
import { SignMessageApprovalCard } from './SignMessageApprovalCard';

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

    case 'signCoinSpends':
      return (
        <SignCoinSpendsApprovalCard approval={approval} appName={appName} />
      );

    case 'signMessage':
      return <SignMessageApprovalCard approval={approval} appName={appName} />;

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

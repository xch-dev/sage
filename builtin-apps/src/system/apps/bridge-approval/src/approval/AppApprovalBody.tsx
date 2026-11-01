import type { RustBridgeApprovalRequest } from 'sage-system-app-sdk';
import { CapabilityGrantApprovalCard } from './CapabilityGrantApprovalCard';
import { CreateOfferApprovalCard } from './CreateOfferApprovalCard';
import { GetSecretKeyApprovalCard } from './GetSecretKeyApprovalCard';
import { NetworkWhitelistGrantApprovalCard } from './NetworkWhitelistGrantApprovalCard';
import { OpenExternalUrlApprovalCard } from './OpenExternalUrlApprovalCard';
import { PermissionGrantsApprovalCard } from './PermissionGrantsApprovalCard';
import { SendXchApprovalCard } from './SendXchApprovalCard';
import { SignCoinSpendsApprovalCard } from './SignCoinSpendsApprovalCard';
import { SignMessageApprovalCard } from './SignMessageApprovalCard';

interface Props {
  approval: RustBridgeApprovalRequest;
  appName: string;
  expanded: boolean;
  working: boolean;
  feeInput: string;
  onFeeInputChange: (value: string) => void;
  handleChangeAcknowledged: boolean;
  onHandleChangeAcknowledgedChange: (value: boolean) => void;
}

export function AppApprovalBody({
  approval,
  appName,
  expanded,
  working,
  feeInput,
  onFeeInputChange,
  handleChangeAcknowledged,
  onHandleChangeAcknowledgedChange,
}: Props) {
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
          working={working}
          feeInput={feeInput}
          onFeeInputChange={onFeeInputChange}
          handleChangeAcknowledged={handleChangeAcknowledged}
          onHandleChangeAcknowledgedChange={onHandleChangeAcknowledgedChange}
        />
      );

    case 'signCoinSpends':
      return (
        <SignCoinSpendsApprovalCard approval={approval} appName={appName} />
      );

    case 'createOffer':
      return <CreateOfferApprovalCard approval={approval} appName={appName} />;

    case 'signMessage':
      return <SignMessageApprovalCard approval={approval} appName={appName} />;

    case 'openExternalUrl':
      return (
        <OpenExternalUrlApprovalCard approval={approval} appName={appName} />
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

    case 'permissionGrants':
      return <PermissionGrantsApprovalCard approval={approval} />;
  }
}

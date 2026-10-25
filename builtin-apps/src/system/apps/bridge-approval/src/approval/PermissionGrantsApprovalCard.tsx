import { PermissionEditor, type RequestedPermissionsView } from 'sage-app-ui';
import type {
  RustBridgeApprovalRequest,
  SageGrantedPermissionsView,
  SageNetworkWhitelistEntry,
} from 'sage-system-app-sdk';

interface Props {
  approval: Extract<RustBridgeApprovalRequest, { kind: 'permissionGrants' }>;
}

export function PermissionGrantsApprovalCard({ approval }: Props) {
  const capabilities = approval.capabilities.map((item) => item.capability);
  const capabilityDefinitions = approval.capabilities.map(
    (item) => item.definition,
  );
  const sharedNetworkWhitelist: SageNetworkWhitelistEntry[] = [];
  const networkWhitelistByNetwork: Record<string, SageNetworkWhitelistEntry[]> =
    {};

  for (const target of approval.networkWhitelist) {
    if (target.networkId) {
      (networkWhitelistByNetwork[target.networkId] ??= []).push(target.entry);
    } else {
      sharedNetworkWhitelist.push(target.entry);
    }
  }

  const requestedPermissions: RequestedPermissionsView = {
    capabilities: {
      required: capabilities,
      optional: [],
    },
    network: {
      whitelist: {
        required: sharedNetworkWhitelist,
        optional: [],
      },
      whitelistByNetwork: Object.fromEntries(
        Object.entries(networkWhitelistByNetwork).map(
          ([networkId, entries]) => [
            networkId,
            { required: entries, optional: [] },
          ],
        ),
      ),
    },
  };

  const grantedPermissions: SageGrantedPermissionsView = {
    capabilities,
    network: {
      whitelist: sharedNetworkWhitelist,
      whitelistByNetwork: networkWhitelistByNetwork,
    },
  };

  return (
    <PermissionEditor
      requestedPermissions={requestedPermissions}
      grantedPermissions={grantedPermissions}
      capabilityDefinitions={capabilityDefinitions}
      editable={false}
      emptyText='No additional permissions requested.'
      requiredSectionTitle='Additional permissions'
    />
  );
}

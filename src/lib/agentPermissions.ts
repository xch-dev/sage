import type {
  SageAgent,
  SageAppCapabilityDefinitionView,
  SageAppWalletScope,
  UserBridgeCapability,
} from '@/bindings';
import type {
  PermissionEditorProps,
  RequestedPermissionsView,
} from 'sage-app-ui';

type GrantedPermissions = PermissionEditorProps['grantedPermissions'];

function definitionKeys(
  definitions: SageAppCapabilityDefinitionView[],
): UserBridgeCapability[] {
  return definitions.map(
    (definition) => definition.key as UserBridgeCapability,
  );
}

export function agentRequestedPermissions(
  definitions: SageAppCapabilityDefinitionView[],
): RequestedPermissionsView {
  return {
    capabilities: { required: [], optional: definitionKeys(definitions) },
  };
}

export function agentGrantedPermissions(
  capabilities: UserBridgeCapability[],
): GrantedPermissions {
  return {
    capabilities,
    network: { whitelist: [], whitelistByNetwork: {} },
  };
}

export function initialAgentCapabilities(
  agent: SageAgent | null,
  definitions: SageAppCapabilityDefinitionView[],
): UserBridgeCapability[] {
  return agent ? agent.capabilities : definitionKeys(definitions);
}

export function defaultAgentWalletScope(
  currentFingerprint?: number,
): SageAppWalletScope {
  return {
    kind: 'selectedWallets',
    fingerprints: currentFingerprint === undefined ? [] : [currentFingerprint],
  };
}

export function hasSelectableWallet(
  scope: SageAppWalletScope,
  wallets: { fingerprint: number }[],
): boolean {
  if (scope.kind === 'allWallets') return true;
  const known = new Set(wallets.map((wallet) => wallet.fingerprint));
  return scope.fingerprints.some((fingerprint) => known.has(fingerprint));
}

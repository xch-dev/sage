import type {
  SageAppCapabilityDefinitionView,
  SageAppPackageManifest,
  SageGrantedPermissionsInput,
  SageGrantedPermissionsView,
  SageNetworkWhitelistEntry,
  UserBridgeCapability,
  UserSageAppView,
} from '@sage-system-app/sdk';
import type { InstallSource } from '../types';

function networkKey(entry: SageNetworkWhitelistEntry): string {
  return `${entry.scheme}://${entry.host}`;
}

function sortCapabilities(
  values: Iterable<UserBridgeCapability>,
): UserBridgeCapability[] {
  return [...new Set(values)].sort((a, b) => a.localeCompare(b));
}

function sortNetwork(
  values: Iterable<SageNetworkWhitelistEntry>,
): SageNetworkWhitelistEntry[] {
  return [...values].sort((a, b) => networkKey(a).localeCompare(networkKey(b)));
}

function definitionMap(definitions: SageAppCapabilityDefinitionView[]) {
  return new Map(
    definitions.map((definition) => [
      definition.key as UserBridgeCapability,
      definition,
    ]),
  );
}

function isUserGrantable(
  definitionsByKey: Map<UserBridgeCapability, SageAppCapabilityDefinitionView>,
  capability: UserBridgeCapability,
): boolean {
  return definitionsByKey.get(capability)?.flags.userGrantable === true;
}

export function emptyGrantedPermissions(): SageGrantedPermissionsInput {
  return {
    capabilities: [],
    network: { whitelist: [] },
  };
}

export function initialGrantedPermissions(
  manifest: SageAppPackageManifest,
  definitions: SageAppCapabilityDefinitionView[],
): SageGrantedPermissionsInput {
  const definitionsByKey = definitionMap(definitions);

  return {
    capabilities: sortCapabilities(
      (manifest.permissions.capabilities.required ?? []).filter((capability) =>
        isUserGrantable(definitionsByKey, capability),
      ),
    ),
    network: {
      whitelist: sortNetwork(
        manifest.permissions.network.whitelist.required ?? [],
      ),
    },
  };
}

export function installManifest(
  source: InstallSource,
): SageAppPackageManifest | null {
  if (source.kind === 'zip') return source.manifest;
  if (source.preview.manifest.kind !== 'full') return null;
  return source.preview.manifest.manifest;
}

export function buildPreviewApp(
  manifest: SageAppPackageManifest,
  grantedPermissions: SageGrantedPermissionsView,
): UserSageAppView {
  return {
    common: {
      identity: {
        id: '__install_preview__',
        originId: '__install_preview__',
      },
      grantedPermissions,
      walletScope: {kind: 'selectedWallets', fingerprints: []},
      activeSnapshot: { manifest },
      icon: null,
    },
    source: { kind: 'zip' },
    pendingUpdate: null,
  };
}

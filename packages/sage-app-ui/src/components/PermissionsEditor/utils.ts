import type {
  SageGrantedPermissionsInput,
  SageGrantedPermissionsView,
  SageNetworkWhitelistEntry,
  UserBridgeCapability,
  SageAppCapabilityDefinitionView,
  SageAppPackageManifest,
} from 'sage-system-app-sdk';

type RequestedWhitelistByNetwork = Record<
  string,
  {
    required?: SageNetworkWhitelistEntry[];
    optional?: SageNetworkWhitelistEntry[];
  }
>;

function sortCapabilities(
  values: Iterable<UserBridgeCapability>,
): UserBridgeCapability[] {
  return [...new Set(values)].sort((a, b) => a.localeCompare(b));
}

export function isUserGrantableCapabilityDefinition(
  definitions: SageAppCapabilityDefinitionView[],
  capability: UserBridgeCapability,
): boolean {
  return (
    definitions.find((definition) => definition.key === capability)?.flags
      .userGrantable === true
  );
}

export function requiredNetworkWhitelistByNetworkInput(
  value: unknown,
): Record<string, SageNetworkWhitelistEntry[]> {
  if (!value || typeof value !== 'object') {
    return {};
  }

  return Object.fromEntries(
    Object.entries(value as RequestedWhitelistByNetwork)
      .map(([networkId, whitelist]) => [
        networkId,
        sortNetworkEntries(whitelist?.required ?? []),
      ])
      .filter(([, entries]) => entries.length > 0),
  );
}

export function initialGrantedPermissionsInput(
  manifest: SageAppPackageManifest,
  definitions: SageAppCapabilityDefinitionView[],
): SageGrantedPermissionsInput {
  return {
    capabilities: sortCapabilities(
      (manifest.permissions.capabilities.required ?? []).filter((capability) =>
        isUserGrantableCapabilityDefinition(definitions, capability),
      ),
    ),
    network: {
      whitelist: sortNetworkEntries(
        manifest.permissions.network.whitelist.required ?? [],
      ),
      whitelistByNetwork: requiredNetworkWhitelistByNetworkInput(
        manifest.permissions.network.whitelistByNetwork,
      ),
    },
  };
}

export function emptyGrantedPermissionsInput(): SageGrantedPermissionsInput {
  return {
    capabilities: [],
    network: {
      whitelist: [],
      whitelistByNetwork: {},
    },
  };
}

export function cn(...classes: Array<string | false | null | undefined>) {
  return classes.filter(Boolean).join(' ');
}

export function networkKey(entry: SageNetworkWhitelistEntry): string {
  return `${entry.scheme}://${entry.host}`;
}

export function sortNetworkEntries(
  entries: SageNetworkWhitelistEntry[],
): SageNetworkWhitelistEntry[] {
  return [...entries].sort((a, b) =>
    networkKey(a).localeCompare(networkKey(b)),
  );
}

export function titleCasePart(value: string): string {
  if (!value) return value;
  return value.charAt(0).toUpperCase() + value.slice(1);
}

export function segmentLabel(segment: string): string {
  return segment.split('_').filter(Boolean).map(titleCasePart).join(' ');
}

export function formatCapabilityLeafLabel(key: string): string {
  const parts = key.split('.');
  return segmentLabel(parts[parts.length - 1] ?? key);
}

export function normalizeKey(key: string): string {
  return key.trim().toLowerCase();
}

export function inputToGrantedPermissionsView(
  permissions: SageGrantedPermissionsInput,
): SageGrantedPermissionsView {
  return {
    capabilities: [...new Set(permissions.capabilities ?? [])].sort((a, b) =>
      a.localeCompare(b),
    ) as UserBridgeCapability[],
    network: {
      whitelist: sortNetworkEntries(permissions.network?.whitelist ?? []),
      whitelistByNetwork: Object.fromEntries(
        Object.entries(permissions.network?.whitelistByNetwork ?? {}).map(
          ([networkId, whitelist]) => [
            networkId,
            sortNetworkEntries(whitelist ?? []),
          ],
        ),
      ),
    },
  };
}

import type {
  SageAppCapabilityDefinitionView,
  SageNetworkWhitelistEntry,
  UserBridgeCapability,
} from '@sage-system-app/sdk';
import type { PermissionEntry } from './types';
import { formatCapabilityLeafLabel, networkKey } from './utils';

export function capabilitySensitivityRank(key: string): number {
  if (key.includes('secret')) return 0;
  if (key === 'storage.persistent_webview') return 2;
  if (key.includes('send') || key.includes('network')) return 3;
  return 4;
}

export function capabilityDefinitionMap(
  definitions: SageAppCapabilityDefinitionView[],
): Map<UserBridgeCapability, SageAppCapabilityDefinitionView> {
  return new Map(
    definitions.map((definition) => [
      definition.key as UserBridgeCapability,
      definition,
    ]),
  );
}

export function isUserGrantableCapability(
  capability: UserBridgeCapability,
  definitionsByKey: Map<UserBridgeCapability, SageAppCapabilityDefinitionView>,
): boolean {
  return definitionsByKey.get(capability)?.flags.userGrantable === true;
}

export function buildCapabilityEntries(
  requestedRequired: UserBridgeCapability[],
  requestedOptional: UserBridgeCapability[],
  grantedCapabilities: UserBridgeCapability[],
  definitionsByKey: Map<UserBridgeCapability, SageAppCapabilityDefinitionView>,
): PermissionEntry[] {
  const grantedSet = new Set<UserBridgeCapability>(grantedCapabilities);

  const requiredEntries: PermissionEntry[] = requestedRequired
    .filter((capability) =>
      isUserGrantableCapability(capability, definitionsByKey),
    )
    .map((capability) => {
      const definition = definitionsByKey.get(capability);
      const key = capability;

      return {
        id: `capability:${key}`,
        kind: 'capability',
        key,
        capability,
        label: definition?.label ?? formatCapabilityLeafLabel(key),
        description: definition?.description ?? null,
        required: true,
        granted: true,
        sensitivityRank: capabilitySensitivityRank(key),
      };
    });

  const optionalEntries: PermissionEntry[] = requestedOptional
    .filter((capability) =>
      isUserGrantableCapability(capability, definitionsByKey),
    )
    .map((capability) => {
      const definition = definitionsByKey.get(capability);
      const key = capability;

      return {
        id: `capability:${key}`,
        kind: 'capability',
        key,
        capability,
        label: definition?.label ?? formatCapabilityLeafLabel(key),
        description: definition?.description ?? null,
        required: false,
        granted: grantedSet.has(capability),
        sensitivityRank: capabilitySensitivityRank(key),
      };
    });

  return [...requiredEntries, ...optionalEntries];
}

export function buildNetworkEntries(
  requestedRequired: SageNetworkWhitelistEntry[],
  requestedOptional: SageNetworkWhitelistEntry[],
  grantedNetworkWhitelist: SageNetworkWhitelistEntry[],
): PermissionEntry[] {
  const grantedSet = new Set(
    grantedNetworkWhitelist.map((entry) => networkKey(entry)),
  );

  const requiredEntries: PermissionEntry[] = requestedRequired.map((entry) => {
    const key = networkKey(entry);

    return {
      id: `network:${key}`,
      kind: 'network',
      key,
      label: key,
      description: null,
      required: true,
      granted: true,
      sensitivityRank: 1,
    };
  });

  const optionalEntries: PermissionEntry[] = requestedOptional.map((entry) => {
    const key = networkKey(entry);

    return {
      id: `network:${key}`,
      kind: 'network',
      key,
      label: key,
      description: null,
      required: false,
      granted: grantedSet.has(key),
      sensitivityRank: 1,
    };
  });

  return [...requiredEntries, ...optionalEntries];
}

export function sortPermissionEntries(
  entries: PermissionEntry[],
): PermissionEntry[] {
  return [...entries].sort((a, b) => {
    if (a.sensitivityRank !== b.sensitivityRank) {
      return a.sensitivityRank - b.sensitivityRank;
    }

    if (a.kind !== b.kind) {
      return a.kind.localeCompare(b.kind);
    }

    return a.key.localeCompare(b.key);
  });
}

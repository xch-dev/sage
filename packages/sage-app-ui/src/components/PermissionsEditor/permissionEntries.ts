import type {
  SageAppCapabilityDefinitionView,
  SageNetworkWhitelistEntry,
  UserBridgeCapability,
} from 'sage-system-app-sdk';
import type {
  NetworkPermissionScheme,
  NetworkPermissionSchemeState,
  PermissionEntry,
} from './types';
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
  section: 'required' | 'optional',
  networkId: string | null = null,
): PermissionEntry[] {
  const requiredKeys = new Set(requestedRequired.map(networkKey));
  const optionalKeys = new Set(requestedOptional.map(networkKey));
  const grantedKeys = new Set(grantedNetworkWhitelist.map(networkKey));
  const hosts = new Set<string>();
  for (const entry of [...requestedRequired, ...requestedOptional]) {
    if (isSupportedNetworkScheme(entry.scheme)) {
      hosts.add(entry.host);
    }
  }
  const entries: PermissionEntry[] = [];

  for (const host of hosts) {
    const httpKey = schemeKey('http', host);
    const httpsKey = schemeKey('https', host);
    const wssKey = schemeKey('wss', host);
    const httpRequested =
      requiredKeys.has(httpKey) || optionalKeys.has(httpKey);
    const httpsRequested =
      requiredKeys.has(httpsKey) || optionalKeys.has(httpsKey);
    const wssRequested = requiredKeys.has(wssKey) || optionalKeys.has(wssKey);
    const hostHasRequired =
      requiredKeys.has(httpKey) ||
      requiredKeys.has(httpsKey) ||
      requiredKeys.has(wssKey);

    if (section === 'required' && !hostHasRequired) continue;
    if (section === 'optional' && hostHasRequired) continue;
    const httpRequired = requiredKeys.has(httpKey);
    const httpGranted = httpRequired || grantedKeys.has(httpKey);
    const wssRequired = requiredKeys.has(wssKey);
    const wssGranted = wssRequired || grantedKeys.has(wssKey);
    const httpsRequired = requiredKeys.has(httpsKey);
    const httpsGranted =
      httpsRequired || wssGranted || grantedKeys.has(httpsKey);
    const httpsVisible = httpsRequested || wssRequested;

    const schemes: Record<
      NetworkPermissionScheme,
      NetworkPermissionSchemeState
    > = {
      http: {
        scheme: 'http',
        key: httpKey,
        required: httpRequired,
        granted: httpGranted,
        disabled: httpRequired,
        visible: httpRequested,
      },
      https: {
        scheme: 'https',
        key: httpsKey,
        required: httpsRequired || wssRequired,
        granted: httpsGranted,
        disabled: httpsRequired || wssGranted,
        visible: httpsVisible,
      },
      wss: {
        scheme: 'wss',
        key: wssKey,
        required: wssRequired,
        granted: wssGranted,
        disabled: wssRequired,
        visible: wssRequested,
      },
    };

    entries.push({
      id:
        networkId === null ? `network:${host}` : `network:${networkId}:${host}`,
      kind: 'network',
      key: networkId === null ? host : `${networkId}:${host}`,
      host,
      networkId,
      label: host,
      description: null,
      required: hostHasRequired,
      granted: httpGranted || httpsGranted || wssGranted,
      sensitivityRank: 1,
      schemes,
    });
  }

  return entries;
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

function isSupportedNetworkScheme(
  scheme: string,
): scheme is NetworkPermissionScheme {
  return scheme === 'http' || scheme === 'https' || scheme === 'wss';
}

function makeNetworkEntry(
  scheme: NetworkPermissionScheme,

  host: string,
): SageNetworkWhitelistEntry {
  return { scheme, host };
}

function schemeKey(scheme: NetworkPermissionScheme, host: string): string {
  return networkKey(makeNetworkEntry(scheme, host));
}

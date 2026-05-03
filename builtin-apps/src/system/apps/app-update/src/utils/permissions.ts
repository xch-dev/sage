import type {
  SageGrantedPermissionsView,
  SageNetworkWhitelistEntry,
  SageRequestedPermissions,
  UserBridgeCapability,
} from '@sage-system-app/sdk';
import { isUserGrantable } from './definitions';

function networkKey(entry: SageNetworkWhitelistEntry) {
  return `${entry.scheme}://${entry.host}`;
}

function sortCapabilities(values: Iterable<UserBridgeCapability>) {
  return [...new Set(values)].sort((a, b) => a.localeCompare(b));
}

function sortNetwork(values: Iterable<SageNetworkWhitelistEntry>) {
  return [...values].sort((a, b) => networkKey(a).localeCompare(networkKey(b)));
}

export function nextPermissionsForUpdate(args: {
  app: any;
  context: any;
  definitionsByKey: Map<any, any>;
}): SageGrantedPermissionsView | null {
  if (!args.context.preview || args.context.preview.manifest.kind !== 'full') {
    return null;
  }

  const nextRequested = args.context.preview.manifest.manifest.permissions;

  const nextCaps = {
    required: nextRequested.capabilities.required ?? [],
    optional: nextRequested.capabilities.optional ?? [],
  };

  const nextNetwork = {
    required: nextRequested.network.whitelist.required ?? [],
    optional: nextRequested.network.whitelist.optional ?? [],
  };

  const nextAllowedCaps = new Set([...nextCaps.required, ...nextCaps.optional]);
  const nextAllowedNetwork = new Set([
    ...nextNetwork.required.map(networkKey),
    ...nextNetwork.optional.map(networkKey),
  ]);

  const retainedCapabilities = (
    args.app.common.grantedPermissions.capabilities ?? []
  )
    .filter((c: UserBridgeCapability) => nextAllowedCaps.has(c))
    .filter((c: UserBridgeCapability) =>
      isUserGrantable(args.definitionsByKey, c),
    );

  const requiredGrantable = nextCaps.required.filter(
    (c: UserBridgeCapability) => isUserGrantable(args.definitionsByKey, c),
  );

  const retainedNetwork = (
    args.app.common.grantedPermissions.network.whitelist ?? []
  ).filter((e: SageNetworkWhitelistEntry) =>
    nextAllowedNetwork.has(networkKey(e)),
  );

  const networkMap = new Map<string, SageNetworkWhitelistEntry>();

  for (const e of retainedNetwork) networkMap.set(networkKey(e), e);
  for (const e of nextNetwork.required) networkMap.set(networkKey(e), e);

  return {
    capabilities: sortCapabilities([
      ...retainedCapabilities,
      ...requiredGrantable,
    ]),
    network: {
      whitelist: sortNetwork(networkMap.values()),
    },
  };
}

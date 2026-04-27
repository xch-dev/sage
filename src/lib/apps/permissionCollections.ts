import type {
  SageNetworkWhitelistEntry,
  UserBridgeCapability,
} from '@/bindings';

export function networkKey(entry: SageNetworkWhitelistEntry): string {
  return `${entry.scheme}://${entry.host}`;
}

export function sortCapabilities(
  values: Iterable<UserBridgeCapability>,
): UserBridgeCapability[] {
  return [...values].sort((a, b) => a.localeCompare(b));
}

export function sortNetwork(
  values: Iterable<SageNetworkWhitelistEntry>,
): SageNetworkWhitelistEntry[] {
  return [...values].sort((a, b) => networkKey(a).localeCompare(networkKey(b)));
}

export function cloneNetwork(
  values: Iterable<SageNetworkWhitelistEntry>,
): SageNetworkWhitelistEntry[] {
  return [...values].map((entry) => ({
    scheme: entry.scheme,
    host: entry.host,
  }));
}

import type { SageNetworkWhitelistEntry } from '@sage-system-app/sdk';

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

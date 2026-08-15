import type { SageAppPackageManifest } from '@sage-system-app/sdk';

export function formatBytes(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes < 0) return 'Unknown';
  if (bytes < 1024) return `${bytes} B`;

  const units = ['KB', 'MB', 'GB', 'TB'];
  let value = bytes / 1024;
  let unitIndex = 0;

  while (value >= 1024 && unitIndex < units.length - 1) {
    value /= 1024;
    unitIndex += 1;
  }

  const digits = value >= 100 ? 0 : value >= 10 ? 1 : 2;
  return `${value.toFixed(digits)} ${units[unitIndex]}`;
}

export function manifestSize(manifest: SageAppPackageManifest): number {
  return manifest.files.reduce((sum, file) => sum + (file.size ?? 0), 0);
}

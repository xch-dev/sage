import type { AppModalIcon } from '@sage-app/ui';
import type { SageAppPackageManifest } from '@sage-system-app/sdk';
import { INSTALL_APP_ICON } from '../constants';
import type { InstallSource } from '../types';

export function resolveManifestIconUrl(
  source: InstallSource,
  manifest: SageAppPackageManifest,
): string | null {
  if (source.kind !== 'url' || !manifest.icon) return null;

  try {
    return new URL(manifest.icon, source.preview.appUrl).toString();
  } catch {
    return null;
  }
}

export function resolveInstallIcon(
  source: InstallSource,
  manifest: SageAppPackageManifest | null,
): AppModalIcon {
  if (!manifest) return INSTALL_APP_ICON;

  return {
    kind: 'url',
    iconUrl: resolveManifestIconUrl(source, manifest),
  };
}

import {
  emptyGrantedPermissionsInput,
  initialGrantedPermissionsInput,
  inputToGrantedPermissionsView,
} from 'sage-app-ui';
import type {
  SageAppCapabilityDefinitionView,
  SageAppPackageManifest,
  SageGrantedPermissionsInput,
  UserSageAppView,
} from 'sage-system-app-sdk';
import type { InstallSource } from '../types';

export const emptyGrantedPermissions = emptyGrantedPermissionsInput;
export const initialGrantedPermissions = initialGrantedPermissionsInput;

export function installManifest(
  source: InstallSource,
): SageAppPackageManifest | null {
  if (source.kind === 'zip') return source.manifest;
  if (source.preview.manifest.kind !== 'full') return null;
  return source.preview.manifest.manifest;
}

export function hasRequiredPermissions(
  manifest: SageAppPackageManifest,
  definitions: SageAppCapabilityDefinitionView[],
): boolean {
  const permissions = initialGrantedPermissions(manifest, definitions);

  return (
    permissions.capabilities.length > 0 ||
    (permissions.network?.whitelist?.length ?? 0) > 0 ||
    Object.values(permissions.network?.whitelistByNetwork ?? {}).some(
      (whitelist) => (whitelist?.length ?? 0) > 0,
    )
  );
}

export function buildPreviewApp(
  manifest: SageAppPackageManifest,
  grantedPermissions: SageGrantedPermissionsInput,
): UserSageAppView {
  return {
    common: {
      identity: {
        id: '__install_preview__',
        originId: '__install_preview__',
      },
      grantedPermissions: inputToGrantedPermissionsView(grantedPermissions),
      walletScope: { kind: 'selectedWallets', fingerprints: [] },
      activeSnapshot: { manifest },
      icon: null,
    },
    source: { kind: 'zip' },
    pendingUpdate: null,
  };
}

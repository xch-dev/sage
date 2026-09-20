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
  const preview =
    source.kind === 'zip' ? source.preview : source.preview.manifest;

  if (preview.kind !== 'full') return null;
  return preview.manifest;
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

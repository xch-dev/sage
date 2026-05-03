import {
  getSageSystemClient,
  type SageGrantedPermissionsInput,
} from '@sage-system-app/sdk';
import type { InstallSource } from './types';

export async function closeSelf() {
  const client = await getSageSystemClient();
  void client.runtimeManager.closeSelf();
}

export async function previewUrl(appUrl: string): Promise<InstallSource> {
  const client = await getSageSystemClient();
  const preview = await client.appInstall.previewUrl({ appUrl });

  return {
    kind: 'url',
    appUrl: preview.appUrl,
    preview,
  };
}

export async function selectAndPreviewZip(): Promise<InstallSource | null> {
  const client = await getSageSystemClient();

  const selected = await client.fileSystem.selectFile({
    title: 'Select Sage app package',
    filters: [{ name: 'Zip Archive', extensions: ['zip'] }],
  });

  if (!selected.path) {
    return null;
  }

  const manifest = await client.appInstall.previewZip({
    zipPath: selected.path,
  });

  return {
    kind: 'zip',
    zipPath: selected.path,
    manifest,
  };
}

export async function installSource(
  source: InstallSource,
  grantedPermissions: SageGrantedPermissionsInput,
) {
  const client = await getSageSystemClient();

  if (source.kind === 'zip') {
    await client.appInstall.installZip({
      zipPath: source.zipPath,
      grantedPermissions,
    });
  } else {
    await client.appInstall.installUrl({
      appUrl: source.appUrl,
      grantedPermissions,
    });
  }
}

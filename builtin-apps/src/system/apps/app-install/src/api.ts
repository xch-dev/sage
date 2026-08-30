import {
  getSageSystemClient,
  type SageGrantedPermissionsInput,
  type SageAppWalletScope,
  type WalletListWalletsResult,
  type AppInstallDownloadProgressEvent,
} from 'sage-system-app-sdk';
import type { InstallSource } from './types';

export async function closeSelf() {
  const client = await getSageSystemClient();
  void client.runtimeManager.closeSelf();
}

export async function listWallets(): Promise<WalletListWalletsResult> {
  const client = await getSageSystemClient();
  return await client.wallet.listWallets();
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
  walletScope: SageAppWalletScope,
  onDownloadProgress?: (event: AppInstallDownloadProgressEvent) => void,
) {
  const client = await getSageSystemClient();
  const unsubscribe = onDownloadProgress
    ? client.appInstall.onDownloadProgress(onDownloadProgress)
    : undefined;

  try {
    if (source.kind === 'zip') {
      await client.appInstall.installZip({
        zipPath: source.zipPath,
        grantedPermissions,
        walletScope,
      });
    } else {
      await client.appInstall.installUrl({
        appUrl: source.appUrl,
        grantedPermissions,
        walletScope,
      });
    }
  } finally {
    unsubscribe?.();
  }
}

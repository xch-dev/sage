import type {
  SageAppPackageManifest,
  SageGrantedPermissions,
  UserSageApp,
} from '@/bindings';
import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { PermissionsEditor } from '@/components/apps/permissions/PermissionsEditor';
import { Globe, Package } from 'lucide-react';
import { InstallSource } from '@/components/apps/InstallAppForm.tsx';
import { AppIconContent } from '@/components/apps/AppIcon.tsx';

interface Props {
  source: InstallSource | null;
  error: string | null;
  installing: boolean;
  grantedPermissions: SageGrantedPermissions;
  onGrantedPermissionsChange: (next: SageGrantedPermissions) => void;
  onCancel: () => void;
  onConfirm: () => void;
}

function buildPreviewApp(
  manifest: SageAppPackageManifest,
  grantedPermissions: SageGrantedPermissions,
): UserSageApp {
  return {
    common: {
      id: '__install_preview__',
      originId: '__install_preview__',
      name: manifest.name,
      version: manifest.version,
      appDir: '',
      entryFile: manifest.entry ?? 'index.html',
      iconFile: manifest.icon ?? 'icon.png',
      requestedPermissions: manifest.permissions ?? {
        network: {
          whitelist: {
            required: [],
            optional: [],
          },
        },
        capabilities: {
          required: [],
          optional: [],
        },
      },
      grantedPermissions,
      capabilityFlags: {
        hasSecretAccess: false,
        hasExternalAccess: false,
        storageMayContainSecrets: false,
        isolated: false,
      },
      storage: { kind: 'unmanaged' },
      activeSnapshot: {
        manifestHash: '__install_preview__',
        snapshotDir: '',
        totalBytes: 0,
        manifest,
      },
    },
    source: { kind: 'zip' },
    pendingUpdate: null,
  };
}

function formatBytes(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes < 0) {
    return 'Unknown';
  }

  if (bytes < 1024) {
    return `${bytes} B`;
  }

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

function resolveInstallIconUrl(
  source: InstallSource,
  manifest: SageAppPackageManifest,
): string | null {
  if (source.kind !== 'url') return null;
  if (!manifest.icon) return null;

  try {
    return new URL(manifest.icon, source.preview.appUrl).toString();
  } catch {
    return null;
  }
}
function computeManifestSize(manifest: SageAppPackageManifest): number {
  return manifest.files.reduce((sum, f) => sum + (f.size ?? 0), 0);
}

function InstallAppIcon({
  source,
  manifest,
}: {
  source: InstallSource;
  manifest: SageAppPackageManifest;
}) {
  return (
    <div className='flex h-16 w-16 items-center justify-center rounded-2xl border bg-muted/30 text-lg font-semibold shadow-sm'>
      <AppIconContent
        name={manifest.name}
        iconUrl={resolveInstallIconUrl(source, manifest)}
      />
    </div>
  );
}

function InstallAppSummary({
  source,
  manifest,
}: {
  source: InstallSource;
  manifest: SageAppPackageManifest;
}) {
  const previewSizeBytes = computeManifestSize(manifest);

  return (
    <div className='rounded-2xl border bg-muted/20 p-4'>
      <div className='flex items-start gap-4'>
        <InstallAppIcon source={source} manifest={manifest} />

        <div className='min-w-0 flex-1'>
          <div className='flex flex-wrap items-center gap-2'>
            <div className='truncate text-xl font-semibold'>
              {manifest.name}
            </div>

            <span className='rounded-full border px-2 py-0.5 text-xs text-muted-foreground'>
              v{manifest.version}
            </span>

            <span className='rounded-full border px-2 py-0.5 text-xs text-muted-foreground'>
              {source.kind === 'url' ? 'URL install' : 'ZIP install'}
            </span>
          </div>

          <div className='mt-3 grid gap-2 text-sm text-muted-foreground sm:grid-cols-2'>
            <div className='flex items-center gap-2'>
              {source.kind === 'url' ? (
                <Globe className='h-4 w-4' />
              ) : (
                <Package className='h-4 w-4' />
              )}
              <span className='truncate'>
                {source.kind === 'url' ? source.appUrl : source.zipPath}
              </span>
            </div>

            <div>
              <span className='text-foreground'>Size:</span>{' '}
              {previewSizeBytes !== null
                ? formatBytes(previewSizeBytes)
                : 'Unknown'}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

export function InstallPermissionsDialog({
  source,
  error,
  installing,
  grantedPermissions,
  onGrantedPermissionsChange,
  onCancel,
  onConfirm,
}: Props) {
  const open = !!source;

  if (!source) {
    return (
      <Dialog open={open} onOpenChange={(nextOpen) => !nextOpen && onCancel()}>
        <DialogContent />
      </Dialog>
    );
  }

  const manifest =
    source.kind === 'zip' ? source.manifest : source.preview.manifest;

  const previewApp = buildPreviewApp(manifest, grantedPermissions);

  return (
    <Dialog open={open} onOpenChange={(nextOpen) => !nextOpen && onCancel()}>
      <DialogContent className='max-w-lg'>
        <DialogHeader className='pb-1'>
          <DialogTitle>Install app</DialogTitle>
        </DialogHeader>

        <div className='space-y-5'>
          <InstallAppSummary source={source} manifest={manifest} />

          <PermissionsEditor
            app={previewApp}
            grantedPermissions={grantedPermissions}
            onGrantedPermissionsChange={onGrantedPermissionsChange}
          />

          {error ? (
            <div className='rounded-lg border border-destructive/30 bg-destructive/5 px-3 py-2 text-sm text-destructive'>
              {error}
            </div>
          ) : null}
        </div>

        <DialogFooter className='gap-2'>
          <Button variant='outline' onClick={onCancel} disabled={installing}>
            Cancel
          </Button>

          <Button onClick={onConfirm} disabled={installing}>
            {installing ? 'Installing...' : 'Install'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

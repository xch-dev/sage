import { AppIcon } from '@sage-app/ui';
import type { SageAppPackageManifest } from '@sage-system-app/sdk';
import type { InstallSource } from '../types';
import { formatBytes, manifestSize } from '../utils/format';
import { resolveInstallIcon } from '../utils/icons';

export function InstallSummary({
  source,
  manifest,
}: {
  source: InstallSource;
  manifest: SageAppPackageManifest;
}) {
  return (
    <div className='rounded-2xl border border-border p-4'>
      <div className='flex items-start gap-4'>
        <div className='h-16 w-16 overflow-hidden rounded-2xl border border-border'>
          <AppIcon appName={manifest.name} appIcon={resolveInstallIcon(source)} />
        </div>

        <div className='min-w-0 flex-1'>
          <div className='truncate text-xl font-semibold'>{manifest.name}</div>

          <div className='mt-1 text-sm text-muted-foreground'>
            v{manifest.version} · {formatBytes(manifestSize(manifest))}
          </div>

          <div className='mt-2 truncate text-xs text-muted-foreground'>
            {source.kind === 'url' ? source.appUrl : source.zipPath}
          </div>
        </div>
      </div>
    </div>
  );
}

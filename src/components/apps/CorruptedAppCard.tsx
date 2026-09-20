import { AppIcon } from '@/components/apps/AppIcon';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { commands, CorruptedInstalledSageApp } from '@/bindings';
import { formatAppError } from '@/lib/apps/formatAppError';
import { RefreshCw, Trash2, TriangleAlert } from 'lucide-react';
import { useState } from 'react';

interface Props {
  app: CorruptedInstalledSageApp;
  onRemove: () => Promise<void>;
}

function appDisplayName(app: CorruptedInstalledSageApp): string {
  return app.manifestHeader?.name ?? app.id;
}

function compatibilityCopy(app: CorruptedInstalledSageApp): {
  badge: string;
  title: string;
  description: string;
} {
  const compatibility = app.compatibility;

  if (!compatibility) {
    return {
      badge: 'unable to load',
      title: 'This app needs attention',
      description:
        'Sage could not read this app’s installed manifest. Check for an update that Sage can load.',
    };
  }

  switch (compatibility.status.kind) {
    case 'compatible': {
      const minimumVersion = app.manifestHeader?.sageVersion?.min;
      return {
        badge: 'needs update',
        title: 'The installed app could not be loaded',
        description: minimumVersion
          ? `This app requires Sage ${minimumVersion} or newer, and you are running Sage ${compatibility.currentVersion}. Its version requirement is compatible, but part of the installed manifest could not be read. Check for an app update that restores compatibility.`
          : `This app’s version requirement is compatible with Sage ${compatibility.currentVersion}, but part of the installed manifest could not be read. Check for an app update that restores compatibility.`,
      };
    }
    case 'requiresNewerSage':
      return {
        badge: 'requires newer Sage',
        title: `Requires Sage ${compatibility.status.minimumVersion} or newer`,
        description: `You are running Sage ${compatibility.currentVersion}. You can still check whether the app publisher has released a compatible update.`,
      };
    case 'untestedNewerSage':
      return {
        badge: 'compatibility warning',
        title: `Tested through Sage ${compatibility.status.testedMaxVersion}`,
        description: `You are running Sage ${compatibility.currentVersion}, and the installed manifest could not be loaded. Check for a newer app version.`,
      };
    case 'invalid':
      return {
        badge: 'invalid requirement',
        title: 'Invalid Sage version requirement',
        description:
          'The compatibility information in this app is invalid. A newer app version may correct it.',
      };
  }
}

export function CorruptedAppCard({ app, onRemove }: Props) {
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  const name = appDisplayName(app);
  const copy = compatibilityCopy(app);

  async function applyUpdate() {
    setBusy(true);
    setMessage(null);

    try {
      await commands.appsRecoverAppUpdate(app.id);
    } catch (err) {
      setMessage(`Update failed: ${formatAppError(err)}`);
    } finally {
      setBusy(false);
    }
  }

  return (
    <Card>
      <CardHeader className='flex flex-row items-start justify-between space-y-0 gap-4'>
        <div className='flex min-w-0 gap-3'>
          <div className='flex h-11 w-11 shrink-0 items-center justify-center overflow-hidden rounded-xl border bg-muted text-sm font-semibold text-muted-foreground'>
            <AppIcon app={{ kind: 'corrupted', ...app }} />
          </div>

          <div className='space-y-2 min-w-0'>
            <CardTitle className='flex flex-wrap items-center gap-2'>
              <TriangleAlert className='h-5 w-5 text-amber-500' />
              <span className='truncate'>{name}</span>
              <Badge variant='outline'>{copy.badge}</Badge>
              {app.manifestHeader ? (
                <Badge variant='outline'>
                  manifest v{app.manifestHeader.manifestVersion}
                </Badge>
              ) : null}
            </CardTitle>

            <div className='text-sm font-medium'>{copy.title}</div>
            <div className='max-w-3xl text-sm text-muted-foreground'>
              {copy.description}
            </div>
          </div>
        </div>

        <div className='flex items-center gap-2 shrink-0'>
          <Button
            variant='default'
            disabled={busy}
            onClick={() => void applyUpdate()}
          >
            <RefreshCw
              className={`h-4 w-4 mr-2 ${busy ? 'animate-spin' : ''}`}
            />
            {busy ? 'Checking…' : 'Repair with update'}
          </Button>

          <Button
            variant='outline'
            disabled={busy}
            onClick={() => void onRemove()}
          >
            <Trash2 className='h-4 w-4 mr-2' />
            Remove
          </Button>
        </div>
      </CardHeader>

      <CardContent className='space-y-3'>
        {message ? <div className='text-sm'>{message}</div> : null}

        <details className='text-xs text-muted-foreground'>
          <summary className='cursor-pointer select-none'>
            Technical details
          </summary>
          <div className='mt-2 break-all'>{app.error}</div>
          <div className='mt-1 break-all'>App ID: {app.id}</div>
        </details>
      </CardContent>
    </Card>
  );
}

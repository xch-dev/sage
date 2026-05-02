import { AppIconContent } from '@/components/apps/AppIcon';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { CorruptedInstalledSageApp } from '@/bindings';
import { Trash2, TriangleAlert } from 'lucide-react';

interface Props {
  app: CorruptedInstalledSageApp;
  onRemove: () => Promise<void>;
}

function appDisplayName(app: CorruptedInstalledSageApp): string {
  return app.manifestHeader?.name ?? app.id;
}

function appIconUrl(app: CorruptedInstalledSageApp): string | null {
  const icon = app.manifestHeader?.icon;
  if (!icon) return null;

  return `sage-app://${app.id}/${icon}`;
}

export function CorruptedAppCard({ app, onRemove }: Props) {
  const name = appDisplayName(app);
  const iconUrl = appIconUrl(app);
  const sageVersion = app.manifestHeader?.sageVersion;

  return (
    <Card>
      <CardHeader className='flex flex-row items-start justify-between space-y-0 gap-4'>
        <div className='flex min-w-0 gap-3'>
          <div className='flex h-11 w-11 shrink-0 items-center justify-center overflow-hidden rounded-xl border bg-muted text-sm font-semibold text-muted-foreground'>
            <AppIconContent name={name} iconUrl={iconUrl} />
          </div>

          <div className='space-y-2 min-w-0'>
            <CardTitle className='flex flex-wrap items-center gap-2'>
              <TriangleAlert className='h-5 w-5 text-destructive' />
              <span className='truncate'>{name}</span>
              <Badge variant='destructive'>corrupted</Badge>
              {app.manifestHeader ? (
                <Badge variant='outline'>
                  manifest v{app.manifestHeader.manifestVersion}
                </Badge>
              ) : null}
            </CardTitle>

            <div className='text-xs text-muted-foreground break-all'>
              ID: {app.id}
            </div>

            <div className='text-xs text-muted-foreground break-all'>
              App dir: {app.appDir}
            </div>

            {sageVersion ? (
              <div className='text-xs text-muted-foreground'>
                Requires Sage {sageVersion.min}
                {sageVersion.testedMax
                  ? ` · tested up to ${sageVersion.testedMax}`
                  : null}
              </div>
            ) : null}
          </div>
        </div>

        <div className='flex items-center gap-2 shrink-0'>
          <Button variant='outline' onClick={() => void onRemove()}>
            <Trash2 className='h-4 w-4 mr-2' />
            Remove
          </Button>
        </div>
      </CardHeader>

      <CardContent>
        <div className='text-sm text-destructive break-words'>{app.error}</div>
      </CardContent>
    </Card>
  );
}

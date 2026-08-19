import { X } from 'lucide-react';

import { AppIcon } from '@/components/apps/AppIcon';
import { Button } from '@/components/ui/button';
import type { AppTaskBarTab } from '@/components/apps/AppTaskBar';

interface Props {
  tabs: AppTaskBarTab[];
  previews: Record<string, string>;
  onSelectApp: (tab: AppTaskBarTab) => void;
  onCloseApp: (tab: AppTaskBarTab) => void;
}

export function AppTabOverview({
  tabs,
  previews,
  onSelectApp,
  onCloseApp,
}: Props) {
  return (
    <div className='h-full overflow-y-auto bg-muted/20 px-4 py-5'>
      <div className='mx-auto max-w-3xl'>
        <div className='mb-4 flex items-baseline justify-between gap-4'>
          <h1 className='text-xl font-semibold'>Open apps</h1>
          <span className='text-sm text-muted-foreground'>
            {tabs.length} open
          </span>
        </div>

        {tabs.length === 0 ? (
          <div className='rounded-xl border border-dashed p-8 text-center text-sm text-muted-foreground'>
            No apps are open yet.
          </div>
        ) : (
          <div className='grid grid-cols-2 gap-3 sm:grid-cols-3'>
            {tabs.map((tab) => {
              const appId = tab.app.common.identity.id;
              const name = tab.app.common.activeSnapshot.manifest.name;
              const preview = previews[appId];

              return (
                <div
                  key={appId}
                  role='button'
                  tabIndex={0}
                  className='group relative overflow-hidden rounded-xl border bg-background text-left shadow-sm transition-transform active:scale-[0.98]'
                  onClick={() => onSelectApp(tab)}
                  onKeyDown={(event) => {
                    if (event.key === 'Enter' || event.key === ' ') {
                      event.preventDefault();
                      onSelectApp(tab);
                    }
                  }}
                >
                  <div className='relative aspect-[3/4] overflow-hidden bg-muted'>
                    {preview ? (
                      <img
                        src={preview}
                        alt=''
                        className='h-full w-full object-cover object-top'
                      />
                    ) : (
                      <div className='flex h-full items-center justify-center'>
                        <div className='flex h-16 w-16 items-center justify-center overflow-hidden rounded-2xl bg-background p-2 shadow-sm'>
                          <AppIcon app={tab.app} />
                        </div>
                      </div>
                    )}

                    <Button
                      type='button'
                      variant='secondary'
                      size='icon'
                      className='absolute right-2 top-2 h-8 w-8 rounded-full shadow'
                      aria-label={`Close ${name}`}
                      onClick={(event) => {
                        event.stopPropagation();
                        onCloseApp(tab);
                      }}
                    >
                      <X className='h-4 w-4' />
                    </Button>
                  </div>

                  <div className='flex items-center gap-2 px-3 py-2.5'>
                    <div className='flex h-5 w-5 shrink-0 items-center justify-center overflow-hidden rounded'>
                      <AppIcon app={tab.app} />
                    </div>
                    <span className='min-w-0 flex-1 truncate text-sm font-medium'>
                      {name}
                    </span>
                  </div>
                </div>
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
}

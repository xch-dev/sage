import type { SageAppView } from '@/bindings';
import type { SandboxLaunchDecision } from '@/lib/apps/sandboxPolicy';
import { AppIcon } from '@/components/apps/AppIcon.tsx';
import React from 'react';

interface Props {
  app: SageAppView;
  launchDecision: SandboxLaunchDecision;
  onOpen: () => void;
  onContextMenu: (event: React.MouseEvent<HTMLDivElement>) => void;
}

export function AppTile({ app, launchDecision, onOpen, onContextMenu }: Props) {
  const isChecking =
    !launchDecision.allowed &&
    launchDecision.title === 'Sandbox tests are still running';

  const isBlocked = !launchDecision.allowed && !isChecking;

  function handleOpen() {
    if (!launchDecision.allowed) return;
    onOpen();
  }

  return (
    <div
      role='button'
      tabIndex={launchDecision.allowed ? 0 : -1}
      onClick={handleOpen}
      onKeyDown={(event) => {
        if (event.key === 'Enter' || event.key === ' ') {
          event.preventDefault();
          handleOpen();
        }
      }}
      onContextMenu={onContextMenu}
      aria-disabled={!launchDecision.allowed}
      className='relative group flex cursor-pointer flex-col items-center gap-3 rounded-2xl p-4 text-center transition-colors hover:bg-muted/50 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring aria-disabled:cursor-default'
    >
      {isChecking || isBlocked ? (
        <div className='absolute inset-0 z-10 flex items-center justify-center rounded-2xl bg-background/55 backdrop-blur-[1px]'>
          {isChecking ? (
            <div className='flex flex-col items-center gap-2'>
              <div className='h-5 w-5 animate-spin rounded-full border-2 border-muted-foreground/30 border-t-muted-foreground' />
              <div className='text-xs text-muted-foreground'>Checking…</div>
            </div>
          ) : (
            <div className='px-3 text-center text-xs font-medium text-amber-600'>
              Blocked
            </div>
          )}
        </div>
      ) : null}

      <div className='flex h-20 w-20 items-center justify-center overflow-hidden rounded-2xl border bg-background shadow-sm'>
        <AppIcon app={app} />
      </div>

      <div className='min-w-0 w-full'>
        <div className='truncate text-sm font-medium'>
          {app.common.activeSnapshot.manifest.name}
        </div>

        {isBlocked ? (
          <div className='relative z-20 mt-1 text-xs text-amber-600'>
            {launchDecision.title}
          </div>
        ) : null}

        {launchDecision.allowed && launchDecision.warning ? (
          <div className='mt-1 text-xs text-amber-600'>
            {launchDecision.title}
          </div>
        ) : null}
      </div>
    </div>
  );
}

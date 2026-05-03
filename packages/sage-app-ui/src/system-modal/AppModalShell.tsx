import type { ReactNode } from 'react';
import { SystemModalShell } from './SystemModalShell';

interface AppModalShellProps {
  title: ReactNode;
  appName: ReactNode;
  children: ReactNode;
  appVersion?: ReactNode;
  appIconSrc?: string | null;
  description?: ReactNode;
  footer?: ReactNode;
  className?: string;
  bodyClassName?: string;
  contentClassName?: string;
}

export function AppModalShell({
  title,
  appName,
  appIconSrc,
  description,
  footer,
  children,
  className = '',
  bodyClassName = '',
  contentClassName = '',
}: AppModalShellProps) {
  return (
    <SystemModalShell
      contentClassName={['p-0 overflow-hidden', contentClassName].join(' ')}
    >
      <div className={['flex h-full min-h-0 flex-col', className].join(' ')}>
        <header className='grid h-20 shrink-0 grid-cols-[5rem_1fr] border-b border-border bg-card'>
          <div className='border-r border-border bg-background'>
            {appIconSrc ? (
              <img
                src={appIconSrc}
                alt=''
                className='h-full w-full object-cover'
              />
            ) : (
              <div className='h-full w-full bg-muted' />
            )}
          </div>

          <div className='flex min-w-0 flex-col justify-center px-5'>
            <div className='truncate text-xs font-medium uppercase tracking-wide text-muted-foreground'>
              {appName}
            </div>

            <h1 className='mt-0.5 truncate text-lg font-semibold text-card-foreground'>
              {title}
            </h1>

            {description ? (
              <div className='mt-0.5 truncate text-sm text-muted-foreground'>
                {description}
              </div>
            ) : null}
          </div>
        </header>

        <main
          className={[
            'min-h-0 flex-1 overflow-auto',
            'bg-card px-6 py-5',
            bodyClassName,
          ].join(' ')}
        >
          {children}
        </main>

        {footer ? (
          <footer className='shrink-0 border-t border-border bg-card px-6 py-4'>
            {footer}
          </footer>
        ) : null}
      </div>
    </SystemModalShell>
  );
}

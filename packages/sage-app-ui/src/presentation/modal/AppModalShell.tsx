import type { ReactNode } from 'react';
import {
  colorWithAlpha,
  resolveModalTint,
  SystemModalShell,
} from './SystemModalShell';
import {
  AppIcon,
} from '../../components';

interface AppModalShellProps {
  title: string;
  appName: string;
  children: ReactNode;
  appIcon?: AppIcon | null;
  description?: string;
  footer?: ReactNode;
  className?: string;
  bodyClassName?: string;
  contentClassName?: string;
  bodyPadded?: boolean;
}

export function AppModalShell({
  title,
  appName,
  appIcon,
  footer,
  children,
  className = '',
  bodyClassName = '',
  contentClassName = '',
  bodyPadded = true,
}: AppModalShellProps) {
  const tint = resolveModalTint();

  return (
    <SystemModalShell
      contentClassName={['p-0 overflow-hidden', contentClassName].join(' ')}
    >
      <div className={['flex h-full min-h-0 flex-col', className].join(' ')}>
        <header className='grid h-16 shrink-0 grid-cols-[4rem_1fr] border-b border-border' style={{
          backgroundColor: colorWithAlpha(tint, 1),
        }}>
          <div className='border-r border-border'>
            <div className='h-full w-full p-1'>
              <AppIcon appName={appName} appIcon={appIcon ?? null} />
            </div>
          </div>

          <div className='flex min-w-0 flex-col justify-center px-3'>
            <div className='truncate text-xs font-medium uppercase tracking-wide text-muted-foreground'>
              {appName}
            </div>

            <h1 className='mt-0.5 truncate text-lg font-semibold text-foreground'>
              {title}
            </h1>
          </div>
        </header>

        <main
          className={[
            'min-h-0 flex-1 overflow-auto',
            bodyPadded ? 'px-6 py-5' : '',
            bodyClassName,
          ].join(' ')}
        >
          {children}
        </main>

        {footer ? (
          <footer className='shrink-0 border-t border-border px-6 py-3'>
            {footer}
          </footer>
        ) : null}
      </div>
    </SystemModalShell>
  );
}

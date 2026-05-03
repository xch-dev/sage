import type { ReactNode } from 'react';
import { SystemModalShell } from './SystemModalShell';
import {
  AppIconBytes,
  AppIconFromBytes,
  AppIconFromUrl,
} from '../../components';
import {SageAppCommonView} from '@sage-system-app/sdk';

interface AppModalShellProps {
  title: string;
  appName: string;
  children: ReactNode;
  appIcon: AppModalIcon;
  description?: string;
  footer?: ReactNode;
  className?: string;
  bodyClassName?: string;
  contentClassName?: string;
}

export type AppModalIcon =
  | { kind: 'url'; iconUrl: string | null }
  | { kind: 'bytes'; icon: AppIconBytes | null };

export function appModalIconFromCommonView(common: SageAppCommonView): AppModalIcon {
  const icon = common.icon;

  if (!icon) {
    return { kind: 'bytes', icon: null };
  }

  return {
    kind: 'bytes',
    icon: {
      bytes: icon.bytes,
      mime: icon.mime,
    },
  };
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
}: AppModalShellProps) {
  return (
    <SystemModalShell
      contentClassName={['p-0 overflow-hidden', contentClassName].join(' ')}
    >
      <div className={['flex h-full min-h-0 flex-col', className].join(' ')}>
        <header className='grid h-16 shrink-0 grid-cols-[4rem_1fr] border-b border-border'>
          <div className='border-r border-border'>
            <div className='h-full w-full p-1'>
              {appIcon.kind === 'url' ? (
                <AppIconFromUrl
                  name={appName}
                  iconUrl={appIcon.iconUrl}
                  className='h-full w-full'
                />
              ) : (
                <AppIconFromBytes
                  name={appName}
                  icon={appIcon.icon}
                  className='h-full w-full'
                />
              )}
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
            'min-h-0 flex-1 overflow-auto px-6 py-5',
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

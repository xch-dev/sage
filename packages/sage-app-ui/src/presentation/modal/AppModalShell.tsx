import {
  type ReactNode,
  useCallback,
  useEffect,
  useRef,
  useState,
} from 'react';
import { platform } from '@tauri-apps/plugin-os';
import { SystemModalShell } from './SystemModalShell';
import { AppIcon } from '../../components';
import { resolveBackgroundTintWithAlpha } from '../utils';

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
  requireScrollEnd?: boolean;
  onScrollEndChange?: (reached: boolean) => void;
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
  requireScrollEnd = false,
  onScrollEndChange,
}: AppModalShellProps) {
  const bodyRef = useRef<HTMLElement | null>(null);
  const [scrolledToEnd, setScrolledToEnd] = useState(!requireScrollEnd);
  const isLinux = platform() === 'linux';

  const updateScrolledToEnd = useCallback(() => {
    if (!requireScrollEnd) {
      setScrolledToEnd(true);
      onScrollEndChange?.(true);
      return;
    }

    const body = bodyRef.current;
    if (!body) return;

    const next =
      body.scrollHeight <= body.clientHeight + 2 ||
      body.scrollTop + body.clientHeight >= body.scrollHeight - 2;

    setScrolledToEnd(next);
    onScrollEndChange?.(next);
  }, [requireScrollEnd, onScrollEndChange]);

  useEffect(() => {
    setScrolledToEnd(!requireScrollEnd);
    onScrollEndChange?.(!requireScrollEnd);

    const frame = requestAnimationFrame(updateScrolledToEnd);

    return () => cancelAnimationFrame(frame);
  }, [requireScrollEnd, updateScrolledToEnd, onScrollEndChange]);

  return (
    <SystemModalShell
      contentClassName={['p-0 overflow-hidden', contentClassName].join(' ')}
    >
      <div
        className={[
          'flex min-h-0 flex-col',
          isLinux ? 'h-full' : 'max-h-[min(620px,72vh)]',
          className,
        ].join(' ')}
      >
        <header
          className='grid h-16 shrink-0 grid-cols-[4rem_1fr] border-b border-border'
          style={{
            backgroundColor: resolveBackgroundTintWithAlpha(1),
          }}
        >
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
          ref={bodyRef}
          onScroll={updateScrolledToEnd}
          className={[
            'min-h-0 overflow-auto',
            isLinux ? 'flex-1' : '',
            bodyPadded ? 'px-6 py-5' : '',
            requireScrollEnd ? 'pb-10' : '',
            bodyClassName,
          ].join(' ')}
        >
          {children}
        </main>

        {footer ? (
          <footer className='relative shrink-0 border-t border-border px-6 py-3'>
            <div
              className={[
                'pointer-events-none absolute inset-x-0 -top-8 flex items-center justify-center transition-opacity duration-200',
                requireScrollEnd && !scrolledToEnd
                  ? 'opacity-100'
                  : 'opacity-0',
              ].join(' ')}
            >
              <span className='animate-bounce text-sm font-medium text-foreground'>
                ↓ Scroll for more ↓
              </span>
            </div>

            {footer}
          </footer>
        ) : null}
      </div>
    </SystemModalShell>
  );
}

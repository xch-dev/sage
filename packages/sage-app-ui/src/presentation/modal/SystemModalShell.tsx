import type { ReactNode } from 'react';
import { resolveBackgroundTintWithAlpha } from '../utils';

interface SystemModalShellProps {
  children: ReactNode;
  className?: string;
  contentClassName?: string;
}

export function SystemModalShell({
  children,
  className = '',
  contentClassName = '',
}: SystemModalShellProps) {
  return (
    <div
      className={[
        'flex h-screen w-screen items-center justify-center overflow-hidden',
        'bg-black/20 text-foreground backdrop-blur-sm',
        className,
      ].join(' ')}
    >
      <div
        className={[
          'max-h-[min(620px,72vh)] w-[min(420px,calc(100vw-4rem))]',
          'overflow-hidden rounded-2xl border border-border shadow-2xl',
          contentClassName,
        ].join(' ')}
        style={{
          backdropFilter: 'blur(80px) saturate(0.55)',
          WebkitBackdropFilter: 'blur(80px) saturate(0.55)',
          backgroundColor: resolveBackgroundTintWithAlpha(),
        }}
      >
        {children}
      </div>
    </div>
  );
}

import type { ReactNode } from 'react';

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
        'h-screen w-screen overflow-hidden bg-black/20 p-6 text-foreground backdrop-blur-sm',
        'flex items-center justify-center',
        className,
      ].join(' ')}
    >
      <div
        className={[
          'h-[min(620px,72vh)] w-[min(620px,calc(100vw-4rem))]',
          'overflow-auto rounded-2xl border border-border p-6 shadow-2xl',
          contentClassName,
        ].join(' ')}
        style={{
          backdropFilter: 'blur(60px) saturate(0.75)',
          WebkitBackdropFilter: 'blur(60px) saturate(0.75)',
          backgroundColor: 'rgb(255 255 255 / 0.12)',
        }}
      >
        {children}
      </div>
    </div>
  );
}

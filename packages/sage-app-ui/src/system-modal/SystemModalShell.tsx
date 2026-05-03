import type { ReactNode } from 'react';

interface SystemModalShellProps {
  children: ReactNode;
  className?: string;
  contentClassName?: string;
}

function resolveModalTint(): string {
  const root = getComputedStyle(document.documentElement);

  const candidates = [
    root.getPropertyValue('--background').trim(),
    root.getPropertyValue('--secondary').trim(),
    root.getPropertyValue('--muted').trim(),
    root.getPropertyValue('--card').trim(),
  ];

  return (
    candidates.find(
      (value) =>
        value.length > 0 &&
        value !== 'transparent' &&
        value !== 'rgba(0, 0, 0, 0)',
    ) ?? '#1d2530'
  );
}

function colorWithAlpha(color: string, alpha: number): string {
  if (color.startsWith('#')) {
    return `color-mix(in srgb, ${color} ${alpha * 100}%, transparent)`;
  }

  if (color.startsWith('rgb') || color.startsWith('hsl')) {
    return `color-mix(in srgb, ${color} ${alpha * 100}%, transparent)`;
  }

  // HSL channel format, e.g. "222 47% 11%"
  return `hsl(${color} / ${alpha})`;
}

export function SystemModalShell({
  children,
  className = '',
  contentClassName = '',
}: SystemModalShellProps) {
  const tint = resolveModalTint();

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
          'h-[min(620px,72vh)] w-[min(620px,calc(100vw-4rem))]',
          'overflow-hidden rounded-2xl border border-border shadow-2xl',
          contentClassName,
        ].join(' ')}
        style={{
          backdropFilter: 'blur(80px) saturate(0.55)',
          WebkitBackdropFilter: 'blur(80px) saturate(0.55)',
          backgroundColor: colorWithAlpha(tint, 0.7),
        }}
      >
        {children}
      </div>
    </div>
  );
}

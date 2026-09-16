import { ReactNode } from 'react';
import { cn } from '@/lib/utils';

interface BannerProps {
  message: string;
  icon?: ReactNode;
  'aria-label'?: string;
  className?: string;
}

export function Banner({
  message,
  icon,
  className,
  'aria-label': ariaLabel,
}: BannerProps) {
  return (
    <div
      className={cn(
        'flex items-center gap-2 px-4 py-1.5 text-xs bg-muted border-b text-muted-foreground',
        className,
      )}
      role='status'
      aria-label={ariaLabel}
      aria-live='polite'
      aria-atomic='true'
    >
      {icon ? icon : null}
      <span>{message}</span>
    </div>
  );
}

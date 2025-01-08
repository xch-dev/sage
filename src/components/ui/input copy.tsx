import * as React from 'react';
import { cn } from '@/lib/utils';
import type { LucideIcon } from 'lucide-react';

export interface InputProps
  extends React.InputHTMLAttributes<HTMLInputElement> {
  startIcon?: LucideIcon;
  endIcon?: LucideIcon;
  startIconProps?: Omit<React.ComponentProps<LucideIcon>, 'ref'>;
  endIconProps?: Omit<React.ComponentProps<LucideIcon>, 'ref'>;
}

const Input = React.forwardRef<HTMLInputElement, InputProps>(
  (
    {
      className,
      type,
      startIcon: StartIcon,
      endIcon: EndIcon,
      startIconProps,
      endIconProps,
      ...props
    },
    ref,
  ) => {
    return (
      <div
        className={cn(
          'relative flex w-full items-center rounded-md border border-neutral-200 bg-transparent shadow-sm transition-colors focus-within:ring-1 focus-within:ring-neutral-950 focus-within:border-neutral-950 dark:border-neutral-800 dark:focus-within:ring-neutral-300',
          className,
        )}
      >
        {StartIcon && (
          <StartIcon
            className={cn(
              'ml-3 h-4 w-4 text-neutral-500 dark:text-neutral-400',
              startIconProps?.className,
            )}
            {...startIconProps}
          />
        )}
        <input
          type={type}
          className={cn(
            'flex h-9 w-full bg-transparent px-3 text-sm file:border-0 file:bg-transparent file:text-sm file:font-medium file:text-neutral-950 placeholder:text-neutral-500 focus-visible:outline-none disabled:cursor-not-allowed disabled:opacity-50 dark:file:text-neutral-50 dark:placeholder:text-neutral-400',
            StartIcon && 'pl-0',
            EndIcon && 'pr-0',
          )}
          ref={ref}
          {...props}
        />
        {EndIcon && (
          <EndIcon
            className={cn(
              'mx-3 h-4 w-4 text-neutral-500 dark:text-neutral-400',
              endIconProps?.className,
            )}
            {...endIconProps}
          />
        )}
      </div>
    );
  },
);

Input.displayName = 'Input';

export { Input };

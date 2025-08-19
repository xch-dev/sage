import * as SwitchPrimitives from '@radix-ui/react-switch';
import * as React from 'react';

import { useTheme } from '@/contexts/ThemeContext';
import { cn } from '@/lib/utils';

const Switch = React.forwardRef<
  React.ElementRef<typeof SwitchPrimitives.Root>,
  React.ComponentPropsWithoutRef<typeof SwitchPrimitives.Root>
>(({ className, ...props }, ref) => {
  const { currentTheme } = useTheme();

  // Get custom switch colors with defaults
  const checkedBg =
    currentTheme?.switches?.checked?.background || 'hsl(var(--primary))';
  const uncheckedBg =
    currentTheme?.switches?.unchecked?.background || 'hsl(var(--border))';

  return (
    <SwitchPrimitives.Root
      className={cn(
        'peer inline-flex h-5 w-9 shrink-0 cursor-pointer items-center rounded-full border-2 border-transparent shadow-sm transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background disabled:cursor-not-allowed disabled:opacity-50',
        className,
      )}
      style={
        {
          '--switch-checked-bg': checkedBg,
          '--switch-unchecked-bg': uncheckedBg,
        } as React.CSSProperties
      }
      {...props}
      ref={ref}
    >
      <SwitchPrimitives.Thumb
        className={cn(
          'pointer-events-none block h-4 w-4 rounded-full bg-background shadow-lg ring-0 transition-transform data-[state=checked]:translate-x-4 data-[state=unchecked]:translate-x-0',
        )}
      />
    </SwitchPrimitives.Root>
  );
});
Switch.displayName = SwitchPrimitives.Root.displayName;

export { Switch };

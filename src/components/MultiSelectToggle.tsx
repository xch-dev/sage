import { cn } from '@/lib/utils';
import { t } from '@lingui/core/macro';
import { CopyPlus } from 'lucide-react';
import { Button } from './ui/button';

export interface MultiSelectToggleProps {
  active: boolean;
  onToggle: () => void;
  disabled?: boolean;
  label?: string;
}

export function MultiSelectToggle({
  active,
  onToggle,
  disabled,
  label,
}: MultiSelectToggleProps) {
  const text = label ?? t`Toggle multi-select`;

  return (
    <Button
      variant='outline'
      size='icon'
      onClick={onToggle}
      aria-label={text}
      aria-pressed={active}
      title={text}
      disabled={disabled}
    >
      <CopyPlus
        className={cn(
          'h-4 w-4',
          active && 'text-green-600 dark:text-green-400',
        )}
        aria-hidden='true'
      />
    </Button>
  );
}

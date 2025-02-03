import { Button } from '@/components/ui/button';
import { CardSize } from '@/hooks/useNftParams';
import { t } from '@lingui/core/macro';
import { Maximize2, Minimize2 } from 'lucide-react';

interface CardSizeToggleProps {
  size: CardSize;
  onChange: (size: CardSize) => void;
}

export function CardSizeToggle({ size, onChange }: CardSizeToggleProps) {
  return (
    <Button
      variant='outline'
      size='icon'
      onClick={() =>
        onChange(size === CardSize.Large ? CardSize.Small : CardSize.Large)
      }
      aria-label={
        size === CardSize.Large
          ? t`Switch to small card size`
          : t`Switch to large card size`
      }
      title={
        size === CardSize.Large
          ? t`Switch to small card size`
          : t`Switch to large card size`
      }
    >
      {size === CardSize.Large ? (
        <Minimize2 className='h-4 w-4' aria-hidden='true' />
      ) : (
        <Maximize2 className='h-4 w-4' aria-hidden='true' />
      )}
    </Button>
  );
}

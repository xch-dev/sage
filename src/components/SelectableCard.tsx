import { cn } from '@/lib/utils';
import { ReactNode } from 'react';

export type SelectionState = [boolean, (value: boolean) => void] | null;

export interface SelectableCardProps {
  selectionState: SelectionState;
  onOpen: () => void;
  className?: string;
  disabled?: boolean;
  ariaLabel?: string;
  children: ReactNode;
}

export function SelectableCard({
  selectionState,
  onOpen,
  className,
  disabled,
  ariaLabel,
  children,
}: SelectableCardProps) {
  const activate = () => {
    if (selectionState === null) {
      onOpen();
    } else {
      selectionState[1](!selectionState[0]);
    }
  };

  return (
    <div
      className={cn(
        'cursor-pointer',
        selectionState?.[0] && 'ring-2 ring-primary ring-offset-2 bg-primary/5',
        className,
      )}
      onClick={activate}
      onKeyDown={(e) => {
        if (e.key === 'Enter' || e.key === ' ') {
          e.preventDefault();
          activate();
        }
      }}
      role='article'
      tabIndex={0}
      aria-label={ariaLabel}
      aria-disabled={disabled}
      aria-selected={selectionState?.[0]}
    >
      {children}
    </div>
  );
}

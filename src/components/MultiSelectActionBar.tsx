import { getFloatingSurfaceStyle } from '@/lib/themeSurface';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { CheckCheck, ChevronDown, X } from 'lucide-react';
import { ReactNode } from 'react';
import { useTheme } from 'theme-o-rama';
import { Button } from './ui/button';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from './ui/dropdown-menu';

export interface MultiSelectActionBarProps {
  selectedCount: number;
  regionAriaLabel: string;
  actionsAriaLabel: string;
  onSelectAll?: () => void;
  onClearSelection?: () => void;
  selectAllAriaLabel?: string;
  children: ReactNode;
}

export function MultiSelectActionBar({
  selectedCount,
  regionAriaLabel,
  actionsAriaLabel,
  onSelectAll,
  onClearSelection,
  selectAllAriaLabel,
  children,
}: MultiSelectActionBarProps) {
  const { currentTheme } = useTheme();

  return (
    <div
      className='absolute flex justify-between items-center gap-2 bottom-6 w-fit max-w-[calc(100vw-2rem)] px-4 p-3 rounded-lg shadow-md shadow-black/20 left-1/2 -translate-x-1/2 bg-card border border-border'
      style={getFloatingSurfaceStyle(currentTheme)}
      role='region'
      aria-label={regionAriaLabel}
    >
      <span
        className='flex-shrink-0 whitespace-nowrap text-card-foreground'
        aria-live='polite'
      >
        <Trans>{selectedCount} selected</Trans>
      </span>
      {(onSelectAll || onClearSelection) && (
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button
              variant='outline'
              size='sm'
              className='flex items-center gap-1 flex-shrink-0'
              aria-label={t`Selection options`}
            >
              <Trans>Select</Trans>
              <ChevronDown className='h-4 w-4' aria-hidden='true' />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align='center'>
            <DropdownMenuGroup>
              {onSelectAll && (
                <DropdownMenuItem
                  className='cursor-pointer'
                  onClick={(e) => {
                    e.stopPropagation();
                    onSelectAll();
                  }}
                  aria-label={selectAllAriaLabel ?? t`Select all`}
                >
                  <CheckCheck className='mr-2 h-4 w-4' aria-hidden='true' />
                  <span>
                    <Trans>Select All</Trans>
                  </span>
                </DropdownMenuItem>
              )}
              {onClearSelection && (
                <DropdownMenuItem
                  className='cursor-pointer'
                  onClick={(e) => {
                    e.stopPropagation();
                    onClearSelection();
                  }}
                  aria-label={t`Clear selection`}
                >
                  <X className='mr-2 h-4 w-4' aria-hidden='true' />
                  <span>
                    <Trans>Clear Selection</Trans>
                  </span>
                </DropdownMenuItem>
              )}
            </DropdownMenuGroup>
          </DropdownMenuContent>
        </DropdownMenu>
      )}
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button
            size='sm'
            className='flex items-center gap-1 flex-shrink-0'
            aria-label={actionsAriaLabel}
          >
            <Trans>Actions</Trans>
            <ChevronDown className='h-5 w-5' aria-hidden='true' />
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align='center'>
          <DropdownMenuGroup>{children}</DropdownMenuGroup>
        </DropdownMenuContent>
      </DropdownMenu>
    </div>
  );
}

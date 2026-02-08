import { t } from '@lingui/core/macro';
import { CaretSortIcon, CheckIcon } from '@radix-ui/react-icons';
import { ChevronLeft, ChevronRight } from 'lucide-react';
import { useCallback, useState } from 'react';

import { cn } from '@/lib/utils';
import { Button } from '../ui/button';
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from '../ui/command';
import { Popover, PopoverContent, PopoverTrigger } from '../ui/popover';

export interface SearchableSelectProps<T> {
  value: string | null | undefined;
  onSelect: (value: string | null) => void;
  items: T[];
  getItemId: (item: T) => string;
  renderItem: (item: T) => React.ReactNode;
  renderSelectedItem?: (item: T | undefined) => React.ReactNode;
  onSearchChange?: (search: string) => void;
  shouldFilter?: boolean;
  onManualInput?: (value: string) => void;
  validateManualInput?: (value: string) => boolean;
  page?: number;
  onPageChange?: (page: number) => void;
  pageSize?: number;
  hasMorePages?: boolean;
  disabled?: (string | null)[];
  isLoading?: boolean;
  className?: string;
  placeholder?: string;
  searchPlaceholder?: string;
  emptyMessage?: string;
}

export function SearchableSelect<T>({
  value,
  onSelect,
  items,
  getItemId,
  renderItem,
  renderSelectedItem,
  onSearchChange,
  shouldFilter = true,
  onManualInput,
  validateManualInput,
  page,
  onPageChange,
  hasMorePages,
  disabled = [],
  isLoading = false,
  className,
  placeholder,
  searchPlaceholder,
  emptyMessage,
}: SearchableSelectProps<T>) {
  const [open, setOpen] = useState(false);
  const [search, setSearch] = useState('');

  const selectedItem = items.find((item) => getItemId(item) === value);

  const handleSelect = useCallback(
    (itemId: string) => {
      onSelect(itemId);
      setOpen(false);
      // Only clear search if it's not a valid manual input
      if (!validateManualInput || !validateManualInput(search)) {
        setSearch('');
      }
    },
    [onSelect, search, validateManualInput],
  );

  const handleSearchChange = useCallback(
    (newSearch: string) => {
      setSearch(newSearch);
      onSearchChange?.(newSearch);

      // Check for manual ID input
      if (validateManualInput && validateManualInput(newSearch)) {
        onManualInput?.(newSearch);
      }

      if (onPageChange && page !== 0) {
        onPageChange(0);
      }
    },
    [validateManualInput, onManualInput, onSearchChange, onPageChange, page],
  );

  const handleOpenChange = useCallback(
    (newOpen: boolean) => {
      setOpen(newOpen);
      if (!newOpen) {
        // Clear search when closing, unless it's a valid manual input
        if (!validateManualInput || !validateManualInput(search)) {
          setSearch('');
        }
      }
    },
    [search, validateManualInput],
  );

  const defaultPlaceholder = t`Select item`;
  const defaultSearchPlaceholder = t`Search...`;
  const defaultEmptyMessage = t`No items found.`;

  const triggerContent = renderSelectedItem
    ? renderSelectedItem(selectedItem)
    : selectedItem
      ? renderItem(selectedItem)
      : (placeholder ?? defaultPlaceholder);

  return (
    <Popover open={open} onOpenChange={handleOpenChange}>
      <PopoverTrigger asChild>
        <Button
          variant='outline'
          role='combobox'
          aria-expanded={open}
          aria-label={placeholder ?? defaultPlaceholder}
          className={cn(
            'flex h-12 w-full items-center justify-between rounded-md border border-input bg-input-background px-3 ring-offset-background truncate font-normal',
            className,
          )}
        >
          <div className='flex items-center h-full text-left truncate flex-1'>
            {triggerContent}
          </div>
          <CaretSortIcon className='ml-2 h-4 w-4 shrink-0 opacity-50' />
        </Button>
      </PopoverTrigger>
      <PopoverContent
        className='p-0'
        style={{ width: 'var(--radix-popover-trigger-width)' }}
        align='start'
        side='bottom'
        collisionPadding={10}
        avoidCollisions={true}
        sticky='partial'
      >
        <Command shouldFilter={shouldFilter}>
          <CommandInput
            placeholder={searchPlaceholder ?? defaultSearchPlaceholder}
            value={search}
            onValueChange={handleSearchChange}
          />

          {onPageChange && (
            <div className='flex items-center justify-between px-2 py-1 border-b'>
              <span className='text-sm text-muted-foreground'>
                Page {(page ?? 0) + 1}
              </span>
              <div className='flex items-center gap-1'>
                <Button
                  variant='ghost'
                  size='icon'
                  className='h-7 w-7'
                  onClick={(e) => {
                    e.preventDefault();
                    onPageChange(Math.max(0, (page ?? 0) - 1));
                  }}
                  disabled={(page ?? 0) === 0}
                >
                  <ChevronLeft className='h-4 w-4' />
                </Button>
                <Button
                  variant='ghost'
                  size='icon'
                  className='h-7 w-7'
                  onClick={(e) => {
                    e.preventDefault();
                    onPageChange((page ?? 0) + 1);
                  }}
                  disabled={!hasMorePages}
                >
                  <ChevronRight className='h-4 w-4' />
                </Button>
              </div>
            </div>
          )}

          <CommandList className='max-h-[200px]'>
            {isLoading ? (
              <div className='py-6 text-center text-sm text-muted-foreground'>
                Loading...
              </div>
            ) : (
              <>
                <CommandEmpty>
                  {emptyMessage ?? defaultEmptyMessage}
                </CommandEmpty>
                <CommandGroup className='p-0'>
                  {items.map((item) => {
                    const itemId = getItemId(item);
                    const isDisabled = disabled.includes(itemId);
                    const isSelected = value === itemId;

                    return (
                      <CommandItem
                        key={itemId}
                        value={itemId}
                        disabled={isDisabled}
                        onSelect={handleSelect}
                        className='px-2 py-1.5 gap-2'
                      >
                        {isSelected && (
                          <CheckIcon className='h-4 w-4 flex-shrink-0' />
                        )}
                        <div className='flex-1 min-w-0'>{renderItem(item)}</div>
                      </CommandItem>
                    );
                  })}
                </CommandGroup>
              </>
            )}
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  );
}

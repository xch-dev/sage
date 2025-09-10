import { t } from '@lingui/core/macro';
import { ChevronLeft, ChevronRight } from 'lucide-react';
import { useMemo } from 'react';
import { Button } from '../ui/button';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '../ui/select';

export interface DropdownSelectorProps<T> {
  loadedItems: T[];
  page: number;
  setPage?: (page: number) => void;
  renderItem: (item: T) => React.ReactNode;
  isSelected: (item: T) => boolean;
  setSelected: (item: T) => void;
  isDisabled?: (item: T) => boolean;
  pageSize?: number;
  className?: string;
  manualInput?: React.ReactNode;
}

export function DropdownSelector<T>({
  loadedItems,
  page,
  setPage,
  renderItem,
  isSelected,
  setSelected,
  isDisabled,
  pageSize = 8,
  className,
  manualInput,
}: DropdownSelectorProps<T>) {
  const foundIndex = useMemo(
    () => loadedItems.findIndex((item) => isSelected(item)),
    [loadedItems, isSelected],
  );

  return (
    <Select
      value={foundIndex === -1 ? undefined : foundIndex.toString()}
      onValueChange={(value) => {
        setSelected(loadedItems[parseInt(value, 10)]);
      }}
    >
      <SelectTrigger
        id='kind'
        aria-label={t`Select asset`}
        className={`flex h-12 max-w-full items-center justify-between rounded-md border border-input bg-input-background px-3 ring-offset-background truncate ${className ?? ''}`}
      >
        <div className='flex items-center h-full text-left truncate'>
          <SelectValue placeholder={t`Select asset`} />
        </div>
      </SelectTrigger>
      <SelectContent
        onKeyDown={(e) => e.stopPropagation()}
        onClick={(e) => e.stopPropagation()}
        style={{ width: 'var(--radix-select-trigger-width)' }}
        hideScrollButtons
      >
        <div className='p-2 space-y-2'>
          {!!setPage && (
            <div className='flex items-center justify-between truncate'>
              <span id='page-label' className='truncate'>
                Page {page + 1}
              </span>
              <div
                className='flex items-center gap-2'
                role='navigation'
                aria-label='Pagination'
              >
                <Button
                  variant='outline'
                  size='icon'
                  onClick={(e) => {
                    e.preventDefault();
                    e.stopPropagation();
                    setPage(Math.max(0, page - 1));
                  }}
                  disabled={page === 0}
                  aria-label='Previous page'
                >
                  <ChevronLeft className='h-4 w-4' aria-hidden='true' />
                </Button>
                <Button
                  variant='outline'
                  size='icon'
                  onClick={(e) => {
                    e.preventDefault();
                    e.stopPropagation();
                    if (loadedItems.length < pageSize) return;
                    setPage(page + 1);
                  }}
                  aria-label='Next page'
                >
                  <ChevronRight className='h-4 w-4' aria-hidden='true' />
                </Button>
              </div>
            </div>
          )}
          {manualInput && (
            <div className='min-w-0 max-w-full truncate'>{manualInput}</div>
          )}
          {(!!setPage || manualInput) && <hr className='my-2' />}
        </div>
        {loadedItems.length === 0 ? (
          <div className='p-4 text-center text-sm text-muted-foreground'>
            No items available
          </div>
        ) : (
          loadedItems.map((item, i) => {
            const disabled = isDisabled?.(item) ?? false;
            return (
              <SelectItem
                value={i.toString()}
                // eslint-disable-next-line react/no-array-index-key
                key={i}
                role='option'
                aria-selected={i === foundIndex}
                aria-disabled={disabled}
                className={`px-2 py-1.5 text-sm rounded-sm truncate ${
                  disabled
                    ? 'opacity-50 cursor-not-allowed'
                    : i === foundIndex
                      ? 'bg-accent cursor-pointer'
                      : 'hover:bg-accent cursor-pointer'
                }`}
              >
                {renderItem(item)}
              </SelectItem>
            );
          })
        )}
      </SelectContent>
    </Select>
  );
}

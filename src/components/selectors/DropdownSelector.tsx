import { t } from '@lingui/core/macro';
import { ChevronLeft, ChevronRight } from 'lucide-react';
import { Button } from '../ui/button';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '../ui/select';

export interface DropdownSelectorProps {
  loadedItems: string[];
  page: number;
  setPage?: (page: number) => void;
  renderItem: (item: string) => React.ReactNode;
  value: string | undefined;
  setValue: (item: string) => void;
  isDisabled?: (item: string) => boolean;
  pageSize?: number;
  className?: string;
  manualInput?: React.ReactNode;
}

export function DropdownSelector({
  loadedItems,
  page,
  setPage,
  renderItem,
  value,
  setValue,
  isDisabled,
  pageSize = 8,
  className,
  manualInput,
}: DropdownSelectorProps) {
  return (
    <Select
      value={value}
      onValueChange={(value) => {
        setValue(value);
      }}
    >
      <SelectTrigger
        id='kind'
        aria-label={t`Select asset`}
        className={`flex h-12 max-w-full items-center justify-between rounded-md border border-input bg-input-background px-3 ring-offset-background truncate ${className ?? ''}`}
      >
        <div className='flex items-center h-full text-left truncate'>
          {value ? (
            renderItem(value)
          ) : (
            <SelectValue placeholder={t`Select asset`} />
          )}
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
            <div
              className='min-w-0 max-w-full truncate'
              onKeyDown={(e) => {
                e.stopPropagation();
              }}
            >
              {manualInput}
            </div>
          )}
          {(!!setPage || manualInput) && <hr className='my-2' />}
        </div>
        {loadedItems.length === 0 ? (
          <div className='p-4 text-center text-sm text-muted-foreground'>
            No items available
          </div>
        ) : (
          loadedItems.map((item) => {
            const disabled = isDisabled?.(item) ?? false;
            return (
              <SelectItem
                value={item}
                key={item}
                role='option'
                aria-selected={item === value}
                aria-disabled={disabled}
                disabled={disabled}
                className={'px-2 py-1.5 text-sm rounded-sm truncate'}
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

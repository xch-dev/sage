import { ChevronDown, ChevronLeft, ChevronRight } from 'lucide-react';
import { PropsWithChildren } from 'react';
import { Button } from '../ui/button';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '../ui/dropdown-menu';

export interface DropdownSelectorProps<T> extends PropsWithChildren {
  totalItems: number;
  loadedItems: T[];
  page: number;
  setPage?: (page: number) => void;
  renderItem: (item: T) => React.ReactNode;
  onSelect: (item: T) => void;
  isDisabled?: (item: T) => boolean;
  pageSize?: number;
  width?: string;
  className?: string;
  manualInput?: React.ReactNode;
}

export function DropdownSelector<T>({
  totalItems,
  loadedItems,
  page,
  setPage,
  renderItem,
  onSelect,
  isDisabled,
  pageSize = 8,
  width = 'w-[300px]',
  className,
  children,
  manualInput,
}: DropdownSelectorProps<T>) {
  const pages = Math.max(1, Math.ceil(totalItems / pageSize));

  return (
    <div className='min-w-0 flex-grow'>
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button
            variant='outline'
            className={`w-full justify-start p-2 h-12 ${className ?? ''}`}
          >
            <div className='flex items-center gap-2 w-full justify-between min-w-0'>
              {children}
              <ChevronDown className='h-4 w-4 opacity-50 mr-2 flex-shrink-0' />
            </div>
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align='start' className={width}>
          <>
            {!!setPage && (
              <DropdownMenuLabel>
                <div className='flex items-center justify-between'>
                  <span>
                    Page {page + 1} / {pages}
                  </span>
                  <div className='flex items-center gap-2'>
                    <Button
                      variant='outline'
                      size='icon'
                      onClick={(e) => {
                        e.preventDefault();
                        setPage(Math.max(0, page - 1));
                      }}
                      disabled={page === 0}
                    >
                      <ChevronLeft className='h-4 w-4' />
                    </Button>
                    <Button
                      variant='outline'
                      size='icon'
                      onClick={(e) => {
                        e.preventDefault();
                        setPage(Math.min(pages - 1, page + 1));
                      }}
                      disabled={page === pages - 1}
                    >
                      <ChevronRight className='h-4 w-4' />
                    </Button>
                  </div>
                </div>
              </DropdownMenuLabel>
            )}
            {manualInput && <div className='p-2'>{manualInput}</div>}
            {(!!setPage || manualInput) && <DropdownMenuSeparator />}
          </>
          <div className='max-h-[260px] overflow-y-auto'>
            {loadedItems.length === 0 ? (
              <div className='p-4 text-center text-sm text-muted-foreground'>
                No items available
              </div>
            ) : (
              loadedItems.map((item, i) => {
                const disabled = isDisabled?.(item) ?? false;
                return (
                  <DropdownMenuItem
                    key={i}
                    onClick={() => onSelect(item)}
                    disabled={disabled}
                    className={disabled ? 'opacity-50 cursor-not-allowed' : ''}
                  >
                    {renderItem(item)}
                  </DropdownMenuItem>
                );
              })
            )}
          </div>
        </DropdownMenuContent>
      </DropdownMenu>
    </div>
  );
}

import { ChevronDown, ChevronLeft, ChevronRight } from 'lucide-react';
import { PropsWithChildren, useEffect, useRef, useState } from 'react';
import { Button } from '../ui/button';

export interface DropdownSelectorProps<T> extends PropsWithChildren {
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
  const [isOpen, setIsOpen] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement | null>(null);

  // Handle click outside and escape key
  useEffect(() => {
    function handleClickOutside(event: MouseEvent) {
      if (
        containerRef.current &&
        !containerRef.current.contains(event.target as Node)
      ) {
        setIsOpen(false);
      }
    }

    function handleKeyDown(event: KeyboardEvent) {
      if (event.key === 'Escape') {
        setIsOpen(false);
      }
    }

    if (isOpen) {
      document.addEventListener('mousedown', handleClickOutside);
      document.addEventListener('keydown', handleKeyDown);
    }

    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
      document.removeEventListener('keydown', handleKeyDown);
    };
  }, [isOpen]);

  // Focus input when dropdown opens
  useEffect(() => {
    if (isOpen && inputRef.current) {
      inputRef.current.focus();
    }
  }, [isOpen]);

  return (
    <div
      className='min-w-0 flex-grow relative'
      role='combobox'
      ref={containerRef}
    >
      <Button
        variant='outline'
        className={`w-full justify-start p-2 h-12 ${className ?? ''}`}
        onClick={() => setIsOpen(!isOpen)}
        aria-expanded={isOpen}
        aria-haspopup='listbox'
      >
        <div className='flex items-center gap-2 w-full justify-between min-w-0'>
          {children}
          <ChevronDown className='h-4 w-4 opacity-50 mr-2 flex-shrink-0' />
        </div>
      </Button>

      {isOpen && (
        <div
          className={`absolute z-50 mt-2 ${width} bg-background border rounded-md shadow-lg`}
          role='listbox'
          aria-label='Options'
        >
          <div className='p-2 space-y-2'>
            {!!setPage && (
              <div className='flex items-center justify-between'>
                <span id='page-label'>Page {page + 1}</span>
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
                ref={(el) => {
                  // Get the input element from the manualInput content
                  if (el) {
                    const input = el.querySelector('input');
                    if (input) {
                      inputRef.current = input;
                    }
                  }
                }}
              >
                {manualInput}
              </div>
            )}
            {(!!setPage || manualInput) && <hr className='my-2' />}
          </div>

          <div className='max-h-[260px] overflow-y-auto'>
            {loadedItems.length === 0 ? (
              <div className='p-4 text-center text-sm text-muted-foreground'>
                No items available
              </div>
            ) : (
              loadedItems.map((item, i) => {
                const disabled = isDisabled?.(item) ?? false;
                return (
                  <div
                    key={i}
                    onClick={() => {
                      if (!disabled) {
                        onSelect(item);
                        setIsOpen(false);
                      }
                    }}
                    role='option'
                    aria-selected={false}
                    aria-disabled={disabled}
                    className={`px-2 py-1.5 text-sm rounded-sm cursor-pointer ${
                      disabled
                        ? 'opacity-50 cursor-not-allowed'
                        : 'hover:bg-accent'
                    }`}
                  >
                    {renderItem(item)}
                  </div>
                );
              })
            )}
          </div>
        </div>
      )}
    </div>
  );
}

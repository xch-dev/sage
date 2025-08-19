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
  const [selectedIndex, setSelectedIndex] = useState(-1);
  const containerRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement | null>(null);
  const listRef = useRef<HTMLDivElement>(null);
  const optionsRef = useRef<(HTMLDivElement | null)[]>([]);

  // Reset selected index when items change
  useEffect(() => {
    setSelectedIndex(-1);
  }, [loadedItems]);

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
      if (!isOpen) return;

      switch (event.key) {
        case 'Escape':
          setIsOpen(false);
          break;
        case 'ArrowDown':
          event.preventDefault();
          setSelectedIndex((prev) =>
            prev < loadedItems.length - 1 ? prev + 1 : prev,
          );
          break;
        case 'ArrowUp':
          event.preventDefault();
          setSelectedIndex((prev) => (prev > 0 ? prev - 1 : prev));
          break;
        case 'Enter':
          event.preventDefault();
          if (selectedIndex >= 0 && selectedIndex < loadedItems.length) {
            // If an item is selected, use that
            const item = loadedItems[selectedIndex];
            if (!isDisabled?.(item)) {
              onSelect(item);
              setIsOpen(false);
              setSelectedIndex(-1);
            }
          } else {
            // If no item is selected, use the first non-disabled item
            const firstValidIndex = loadedItems.findIndex(
              (item) => !isDisabled?.(item),
            );
            if (firstValidIndex >= 0) {
              onSelect(loadedItems[firstValidIndex]);
              setIsOpen(false);
              setSelectedIndex(-1);
            }
          }
          break;
        case 'Tab':
          if (event.shiftKey) {
            // Allow shift+tab to move focus backwards
            return;
          }
          // If we're on the input, let the next tab go to the list
          if (document.activeElement === inputRef.current) {
            event.preventDefault();
            listRef.current?.focus();
            setSelectedIndex(0);
          }
          break;
      }
    }

    document.addEventListener('mousedown', handleClickOutside);
    document.addEventListener('keydown', handleKeyDown);

    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
      document.removeEventListener('keydown', handleKeyDown);
    };
  }, [isOpen, loadedItems, selectedIndex, onSelect, isDisabled]);

  // Scroll selected item into view
  useEffect(() => {
    if (selectedIndex >= 0 && optionsRef.current[selectedIndex]) {
      optionsRef.current[selectedIndex]?.scrollIntoView({
        block: 'nearest',
      });
    }
  }, [selectedIndex]);

  // Focus input when dropdown opens
  useEffect(() => {
    if (isOpen && inputRef.current) {
      inputRef.current.focus();
    }
  }, [isOpen]);

  // Add new effect to handle search text changes
  useEffect(() => {
    if (!setPage || !inputRef.current) return;

    const handleInput = () => {
      if (page !== 0) {
        setTimeout(() => {
          setPage(0);
        }, 0);
      }
    };

    inputRef.current.addEventListener('input', handleInput);
    return () => {
      inputRef.current?.removeEventListener('input', handleInput);
    };
  }, [setPage, page, inputRef]);

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
          className={`absolute mt-2 ${width} bg-popover border rounded-md shadow-lg`}
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
                      e.stopPropagation();
                      setPage(Math.max(0, page - 1));
                      inputRef.current?.focus();
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
                      inputRef.current?.focus();
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

          <div
            className='max-h-[260px] overflow-y-auto'
            ref={listRef}
            tabIndex={0}
            role='listbox'
          >
            {loadedItems.length === 0 ? (
              <div className='p-4 text-center text-sm text-muted-foreground'>
                No items available
              </div>
            ) : (
              loadedItems.map((item, i) => {
                const disabled = isDisabled?.(item) ?? false;
                return (
                  <div
                    // eslint-disable-next-line react/no-array-index-key
                    key={i}
                    ref={(el) => (optionsRef.current[i] = el)}
                    onClick={() => {
                      if (!disabled) {
                        onSelect(item);
                        setIsOpen(false);
                      }
                    }}
                    role='option'
                    aria-selected={i === selectedIndex}
                    aria-disabled={disabled}
                    className={`px-2 py-1.5 text-sm rounded-sm cursor-pointer ${
                      disabled
                        ? 'opacity-50 cursor-not-allowed'
                        : i === selectedIndex
                          ? 'bg-accent'
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

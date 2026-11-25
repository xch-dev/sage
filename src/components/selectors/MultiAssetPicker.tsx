import { Button } from '@/components/ui/button';
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from '@/components/ui/command';
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from '@/components/ui/popover';
import { cn } from '@/lib/utils';
import { Trans } from '@lingui/react/macro';
import { CheckIcon, Loader2 } from 'lucide-react';
import { ReactNode, useEffect, useState } from 'react';

const pageSize = 20;

export interface PickerPage<T> {
  items: T[];
  hasMore: boolean;
}

export interface MultiAssetPickerProps<T> {
  /** Assets already in the offer, shown checked. */
  selected: string[];
  /** Adds the asset to the offer, or removes it if it's already there. */
  onToggle: (id: string) => void;
  /** Fetches the first `limit` assets matching the search. */
  load: (query: string, limit: number) => Promise<PickerPage<T>>;
  getId: (item: T) => string;
  /** The item's content, after its checkbox. */
  renderItem: (item: T) => ReactNode;
  searchPlaceholder: string;
  emptyMessage: ReactNode;
  /** Whether the search is a pasted ID, which gets `idNotFoundMessage`. */
  isId: (query: string) => boolean;
  idNotFoundMessage: ReactNode;
  /** How many are in the offer, shown next to the Done button. */
  countLabel: (count: number) => string;
  /** The button that opens the picker. */
  children: ReactNode;
}

/**
 * A searchable list of assets that opens from a button. It stays open so that
 * several can be added in one go; picking one again removes it.
 */
export function MultiAssetPicker<T>({
  selected,
  onToggle,
  load,
  getId,
  renderItem,
  searchPlaceholder,
  emptyMessage,
  isId,
  idNotFoundMessage,
  countLabel,
  children,
}: MultiAssetPickerProps<T>) {
  const [open, setOpen] = useState(false);
  const [search, setSearch] = useState('');
  const [limit, setLimit] = useState(pageSize);
  const [items, setItems] = useState<T[]>([]);
  const [hasMore, setHasMore] = useState(false);
  const [isLoading, setIsLoading] = useState(false);

  const query = search.trim();

  useEffect(() => {
    if (!open) return;
    let cancelled = false;

    // Wait for typing to pause before searching.
    const timeout = setTimeout(
      () => {
        setIsLoading(true);
        load(query, limit)
          .then((page) => {
            if (cancelled) return;
            setItems(page.items);
            setHasMore(page.hasMore);
          })
          .catch(() => {
            if (!cancelled) setItems([]);
          })
          .finally(() => {
            if (!cancelled) setIsLoading(false);
          });
      },
      query ? 250 : 0,
    );

    return () => {
      cancelled = true;
      clearTimeout(timeout);
    };
  }, [open, query, limit, load]);

  const handleOpenChange = (next: boolean) => {
    setOpen(next);
    if (!next) {
      setSearch('');
      setLimit(pageSize);
    }
  };

  return (
    <Popover open={open} onOpenChange={handleOpenChange}>
      <PopoverTrigger asChild>{children}</PopoverTrigger>
      <PopoverContent
        className='w-[min(26rem,calc(100vw-2rem))] p-0'
        align='start'
      >
        <Command shouldFilter={false}>
          <CommandInput
            placeholder={searchPlaceholder}
            value={search}
            onValueChange={(value) => {
              setSearch(value);
              setLimit(pageSize);
            }}
          />
          <CommandList className='max-h-80'>
            {!isLoading && (
              <CommandEmpty>
                {isId(query) ? idNotFoundMessage : emptyMessage}
              </CommandEmpty>
            )}
            {items.length > 0 && (
              <CommandGroup>
                {items.map((item) => {
                  const id = getId(item);
                  const checked = selected.includes(id);

                  return (
                    <CommandItem
                      key={id}
                      value={id}
                      onSelect={() => onToggle(id)}
                      className='gap-3'
                      aria-checked={checked}
                    >
                      <span
                        aria-hidden='true'
                        className={cn(
                          'flex h-4 w-4 shrink-0 items-center justify-center rounded-sm border',
                          checked
                            ? 'border-primary bg-primary text-primary-foreground'
                            : 'border-muted-foreground',
                        )}
                      >
                        {checked && <CheckIcon className='h-3 w-3' />}
                      </span>
                      {renderItem(item)}
                    </CommandItem>
                  );
                })}
              </CommandGroup>
            )}
            {isLoading && (
              <div className='flex items-center justify-center gap-2 py-4 text-sm text-muted-foreground'>
                <Loader2 className='h-4 w-4 animate-spin' aria-hidden='true' />
                <Trans>Loading…</Trans>
              </div>
            )}
            {!isLoading && hasMore && (
              <div className='p-1'>
                <Button
                  type='button'
                  variant='ghost'
                  size='sm'
                  className='w-full'
                  onClick={() => setLimit(limit + pageSize)}
                >
                  <Trans>Show more</Trans>
                </Button>
              </div>
            )}
          </CommandList>
        </Command>
        <div className='flex items-center justify-between gap-2 border-t px-3 py-2'>
          <span className='text-xs text-muted-foreground'>
            {countLabel(selected.length)}
          </span>
          <Button
            type='button'
            size='sm'
            onClick={() => handleOpenChange(false)}
          >
            <Trans>Done</Trans>
          </Button>
        </div>
      </PopoverContent>
    </Popover>
  );
}

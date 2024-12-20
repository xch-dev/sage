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
  pageSize?: number;
  totalItems: number;
  loadedItems: T[];
  page: number;
  setPage: (page: number) => void;
  renderItem: (item: T) => React.ReactNode;
  onSelect: (item: T) => void;
  isDisabled: (item: T) => boolean;
}

export function DropdownSelector<T>(props: DropdownSelectorProps<T>) {
  const pageSize = props.pageSize ?? 8;
  const pages = Math.max(1, Math.ceil(props.totalItems / pageSize));
  const { page, setPage } = props;

  return (
    <div className='min-w-0 flex-grow'>
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button
            variant='outline'
            className='w-full justify-start rounded-r-none p-2 h-12'
          >
            <div className='flex items-center gap-2 w-full justify-between min-w-0'>
              {props.children}
              <ChevronDown className='h-4 w-4 opacity-50 mr-2 flex-shrink-0' />
            </div>
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align='start' className='w-[300px]'>
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
          <DropdownMenuSeparator />
          <div className='max-h-[300px] overflow-y-auto'>
            {props.loadedItems.length === 0 ? (
              <div className='p-4 text-center text-sm text-muted-foreground'>
                No items available
              </div>
            ) : (
              props.loadedItems.map((item, i) => {
                const isDisabled = props.isDisabled(item);

                return (
                  <DropdownMenuItem
                    key={i}
                    onClick={() => props.onSelect(item)}
                    disabled={isDisabled}
                    className={
                      isDisabled ? 'opacity-50 cursor-not-allowed' : ''
                    }
                  >
                    {props.renderItem(item)}
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

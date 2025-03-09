import { t } from '@lingui/core/macro';
import {
  ArrowDownAz,
  ArrowUpAz,
  SearchIcon,
  Settings2,
  XIcon,
  ListFilter,
  List,
} from 'lucide-react';
import { Button } from './ui/button';
import { Input } from './ui/input';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from './ui/dropdown-menu';
import {
  TransactionParams,
  SetTransactionParams,
} from '@/hooks/useTransactionsParams';
import { motion, AnimatePresence } from 'framer-motion';
import { useDebounce } from '@/hooks/useDebounce';
import { useState, useEffect } from 'react';

const optionsPaginationVariants = {
  enter: { opacity: 1, y: 0 },
  exit: { opacity: 0, y: 20, transition: { duration: 0.15 } },
};

interface TransactionOptionsProps {
  params: TransactionParams;
  onParamsChange: SetTransactionParams;
  total: number;
  isLoading?: boolean;
  className?: string;
  renderPagination: () => React.ReactNode;
}

export function TransactionOptions({
  params,
  onParamsChange,
  className,
  renderPagination,
}: TransactionOptionsProps) {
  const { search, ascending, summarized } = params;
  const [searchValue, setSearchValue] = useState(search);
  const debouncedSearch = useDebounce(searchValue, 400);

  useEffect(() => {
    setSearchValue(search);
  }, [search]);

  useEffect(() => {
    if (debouncedSearch !== search) {
      onParamsChange({ search: debouncedSearch, page: 1 });
    }
  }, [debouncedSearch, search, onParamsChange]);

  return (
    <div
      className={`flex flex-col gap-4 ${className}`}
      role='toolbar'
      aria-label={t`Transaction filtering and sorting options`}
    >
      <div className='relative flex-1' role='search'>
        <div className='relative'>
          <SearchIcon
            className='absolute left-2 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground'
            aria-hidden='true'
          />
          <Input
            value={searchValue}
            aria-label={t`Search transactions...`}
            title={t`Search transactions...`}
            placeholder={t`Search transactions...`}
            onChange={(e) => setSearchValue(e.target.value)}
            className='w-full pl-8 pr-8'
          />
        </div>
        {searchValue && (
          <Button
            variant='ghost'
            size='icon'
            title={t`Clear search`}
            aria-label={t`Clear search`}
            className='absolute right-0 top-0 h-full px-2 hover:bg-transparent'
            onClick={() => setSearchValue('')}
          >
            <XIcon className='h-4 w-4' aria-hidden='true' />
          </Button>
        )}
      </div>

      <div className='flex items-center justify-between'>
        <AnimatePresence mode='wait'>
          <motion.div
            key='pagination'
            initial={{ opacity: 0, y: 20 }}
            animate={optionsPaginationVariants.enter}
            exit={optionsPaginationVariants.exit}
          >
            {renderPagination()}
          </motion.div>
        </AnimatePresence>
        <div className='flex gap-2'>
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button
                variant='outline'
                size='icon'
                aria-label={t`View options`}
                title={t`View options`}
              >
                <Settings2 className='h-4 w-4' aria-hidden='true' />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align='end'>
              <DropdownMenuItem
                className='cursor-pointer'
                onClick={() => onParamsChange({ ascending: !ascending })}
              >
                {ascending ? (
                  <ArrowDownAz className='mr-2 h-4 w-4' aria-hidden='true' />
                ) : (
                  <ArrowUpAz className='mr-2 h-4 w-4' aria-hidden='true' />
                )}
                {ascending ? t`Sort Descending` : t`Sort Ascending`}
              </DropdownMenuItem>
              <DropdownMenuSeparator />
              <DropdownMenuItem
                className='cursor-pointer'
                onClick={() => onParamsChange({ summarized: !summarized })}
              >
                {summarized ? (
                  <ListFilter className='mr-2 h-4 w-4' aria-hidden='true' />
                ) : (
                  <List className='mr-2 h-4 w-4' aria-hidden='true' />
                )}
                {summarized ? t`Detailed View` : t`Summarized View`}
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
      </div>
    </div>
  );
}

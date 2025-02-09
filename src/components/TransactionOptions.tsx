import { t } from '@lingui/core/macro';
import {
  ArrowDownAz,
  ArrowUpAz,
  SearchIcon,
  Settings2,
  XIcon,
} from 'lucide-react';
import { Button } from './ui/button';
import { Input } from './ui/input';
import { Pagination } from './Pagination';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from './ui/dropdown-menu';
import {
  TransactionParams,
  SetTransactionParams,
} from '@/hooks/useTransactionsParams';

interface TransactionOptionsProps {
  params: TransactionParams;
  onParamsChange: SetTransactionParams;
  total: number;
  isLoading?: boolean;
  className?: string;
}

export function TransactionOptions({
  params,
  onParamsChange,
  total,
  isLoading,
  className,
}: TransactionOptionsProps) {
  const { search, page, pageSize, ascending } = params;

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
            value={search}
            aria-label={t`Search transactions...`}
            title={t`Search transactions...`}
            placeholder={t`Search transactions...`}
            onChange={(e) => onParamsChange({ search: e.target.value })}
            className='w-full pl-8 pr-8'
          />
        </div>
        {search && (
          <Button
            variant='ghost'
            size='icon'
            title={t`Clear search`}
            aria-label={t`Clear search`}
            className='absolute right-0 top-0 h-full px-2 hover:bg-transparent'
            onClick={() => onParamsChange({ search: '' })}
          >
            <XIcon className='h-4 w-4' aria-hidden='true' />
          </Button>
        )}
      </div>

      <div className='flex items-center justify-between'>
        <Pagination
          page={page}
          total={total}
          pageSize={pageSize}
          onPageChange={(newPage) => onParamsChange({ page: newPage })}
          onPageSizeChange={(newSize) => onParamsChange({ pageSize: newSize, page: 1 })}
          pageSizeOptions={[10, 25, 50, 100]}
          compact={true}
          isLoading={isLoading}
        />
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
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
      </div>
    </div>
  );
}

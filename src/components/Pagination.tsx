import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { Button } from './ui/button';
import { ChevronRightIcon, ChevronLeftIcon } from '@radix-ui/react-icons';

type PaginationProps = {
  page: number;
  total?: number;
  canLoadMore?: boolean;
  isLoading?: boolean;
  onPageChange: (page: number) => void;
  pageSize: number;
  onPageSizeChange: (pageSize: number) => void;
  pageSizeOptions?: number[];
};

export const Pagination = ({
  page,
  total,
  canLoadMore,
  isLoading,
  onPageChange,
  pageSize,
  onPageSizeChange,
  pageSizeOptions = [8, 16, 32, 64],
}: PaginationProps) => {
  const totalPages = total
    ? Math.max(1, Math.ceil(total / pageSize))
    : undefined;
  const isFirstPage = page === 1;
  const isLastPage = totalPages
    ? page === totalPages
    : !canLoadMore || isLoading;

  return (
    <div className='flex justify-between gap-2'>
      <div className='flex items-center justify-start gap-3'>
        <Button
          size='icon'
          variant='outline'
          onClick={() => {
            if (isFirstPage) return;
            onPageChange(Math.max(1, page - 1));
          }}
          disabled={isFirstPage}
          title={t`Previous page`}
        >
          <ChevronLeftIcon className='h-4 w-4' />
        </Button>

        {totalPages ? (
          <Select
            onValueChange={(value) => onPageChange(parseInt(value))}
            defaultValue={page.toString()}
            value={page.toString()}
            disabled={totalPages === 1}
          >
            <SelectTrigger className='w-min text-sm' title={t`Page`}>
              <SelectValue placeholder={t`Page`}>
                {page} / {totalPages}
              </SelectValue>
            </SelectTrigger>
            <SelectContent>
              {Array.from({ length: totalPages }, (_, i) => (
                <SelectItem key={i} value={(i + 1).toString()}>
                  {i + 1}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        ) : (
          <div className='text-sm text-muted-foreground font-medium  text-center'>
            <Trans>Page {page}</Trans>
          </div>
        )}

        <Button
          size='icon'
          variant='outline'
          onClick={() => {
            if (isLastPage) return;
            onPageChange(Math.min(totalPages ?? Infinity, page + 1));
          }}
          disabled={isLastPage}
          title={t`Next page`}
        >
          <ChevronRightIcon className='h-4 w-4' />
        </Button>
      </div>

      <Select
        onValueChange={(value) => {
          onPageChange(1);
          onPageSizeChange(parseInt(value));
        }}
        defaultValue={pageSize.toString()}
        value={pageSize.toString()}
      >
        <SelectTrigger 
          className='w-min' 
          title={t`Items per page`}
          aria-label={t`Select number of items per page`}
        >
          {pageSize}
        </SelectTrigger>
        <SelectContent>
          <SelectGroup>
            {pageSizeOptions.map((size) => (
              <SelectItem key={size} value={size.toString()}>
                {size}
              </SelectItem>
            ))}
          </SelectGroup>
        </SelectContent>
      </Select>
    </div>
  );
};

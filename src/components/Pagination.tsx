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
  compact?: boolean;
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
  compact = false,
}: PaginationProps) => {
  const totalPages = total
    ? Math.max(1, Math.ceil(total / pageSize))
    : undefined;
  const isFirstPage = page === 1;
  const isLastPage = totalPages
    ? page === totalPages
    : !canLoadMore || isLoading;

  return (
    <nav
      role='navigation'
      aria-label={t`Pagination`}
      className='flex justify-between gap-2'
    >
      <div className={`flex items-center justify-start gap-2`}>
        <Button
          size='icon'
          variant='outline'
          onClick={() => {
            if (isFirstPage) return;
            onPageChange(Math.max(1, page - 1));
          }}
          disabled={isFirstPage}
          aria-label={t`Go to previous page`}
          aria-disabled={isFirstPage}
        >
          <ChevronLeftIcon className='h-4 w-4' aria-hidden='true' />
        </Button>

        {totalPages ? (
          <Select
            onValueChange={(value) => onPageChange(parseInt(value))}
            defaultValue={page.toString()}
            value={page.toString()}
            disabled={totalPages === 1}
            aria-label={t`Select page number`}
          >
            <SelectTrigger
              className='w-min text-sm'
              aria-label={t`Current page ${page} of ${totalPages}`}
            >
              <SelectValue>
                {page} / {totalPages}
              </SelectValue>
            </SelectTrigger>
            <SelectContent>
              {Array.from({ length: totalPages }, (_, i) => (
                <SelectItem
                  key={i}
                  value={(i + 1).toString()}
                  aria-label={t`Go to page ${i + 1}`}
                >
                  {i + 1}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        ) : (
          <div
            className='text-sm text-muted-foreground font-medium text-center'
            aria-live='polite'
            aria-atomic='true'
          >
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
          aria-label={t`Go to next page`}
          aria-disabled={isLastPage}
        >
          <ChevronRightIcon className='h-4 w-4' aria-hidden='true' />
        </Button>
      </div>

      {!compact && (
        <div className='flex items-center gap-2'>
          <label id='items-per-page-label' className='sr-only'>
            <Trans>Items per page</Trans>
          </label>
          <Select
            onValueChange={(value) => {
              onPageSizeChange(parseInt(value));
            }}
            defaultValue={pageSize.toString()}
            value={pageSize.toString()}
            aria-labelledby='items-per-page-label'
          >
            <SelectTrigger
              className='w-min'
              aria-label={t`${pageSize} items per page`}
            >
              {pageSize}
            </SelectTrigger>
            <SelectContent>
              <SelectGroup>
                {pageSizeOptions.map((size) => (
                  <SelectItem
                    key={size}
                    value={size.toString()}
                    aria-label={t`Show ${size} items per page`}
                  >
                    {size}
                  </SelectItem>
                ))}
              </SelectGroup>
            </SelectContent>
          </Select>

          {total && total > 0 ? (
            <span
              className='hidden sm:inline-block text-sm text-muted-foreground'
              aria-label={t`Total items: ${total}`}
            >
              <Trans>Total: {total}</Trans>
            </span>
          ) : null}
        </div>
      )}
    </nav>
  );
};

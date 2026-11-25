import { t } from '@lingui/core/macro';
import { Button } from './ui/button';
import { ChevronLeft, ChevronRight } from 'lucide-react';
import { Select, SelectContent, SelectItem, SelectTrigger } from './ui/select';

interface SimplePaginationProps {
  currentPage: number;
  pageCount: number;
  setCurrentPage: (page: number) => void;
  className?: string;
  size?: 'default' | 'sm';
  align?: 'start' | 'end' | 'between';
  actions?: React.ReactNode;
  pageSize?: number;
  onPageSizeChange?: (pageSize: number) => void;
  pageSizeOptions?: number[];
}

export function SimplePagination({
  currentPage,
  pageCount,
  setCurrentPage,
  className = '',
  size = 'default',
  align = 'between',
  actions,
  pageSize,
  onPageSizeChange,
  pageSizeOptions = [10, 25, 50, 100],
}: SimplePaginationProps) {
  const justifyClass =
    align === 'start'
      ? 'justify-start'
      : align === 'end'
        ? 'justify-end'
        : 'justify-between';

  return (
    <div className={`flex items-center ${justifyClass} ${className}`}>
      {align === 'between' && actions && (
        <div className='flex space-x-2'>{actions}</div>
      )}
      <div className='flex space-x-2'>
        {pageSize !== undefined && onPageSizeChange && (
          <Select
            value={pageSize.toString()}
            onValueChange={(value) => onPageSizeChange(parseInt(value))}
          >
            <SelectTrigger
              className={`w-min gap-2 ${size === 'sm' ? 'h-8 text-xs' : ''}`}
              aria-label={t`${pageSize} items per page`}
            >
              {pageSize}
            </SelectTrigger>
            <SelectContent>
              {pageSizeOptions.map((size) => (
                <SelectItem
                  key={size}
                  value={size.toString()}
                  aria-label={t`Show ${size} items per page`}
                >
                  {size}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        )}
        <Button
          variant='outline'
          size={size}
          onClick={() => setCurrentPage(Math.max(0, currentPage - 1))}
          disabled={currentPage === 0}
          aria-label={t`Previous page`}
        >
          <ChevronLeft className='h-4 w-4' aria-hidden='true' />
        </Button>
        <Button
          variant='outline'
          size={size}
          onClick={() =>
            setCurrentPage(Math.min(pageCount - 1, currentPage + 1))
          }
          disabled={currentPage >= pageCount - 1}
          aria-label={t`Next page`}
        >
          <ChevronRight className='h-4 w-4' aria-hidden='true' />
        </Button>
      </div>
    </div>
  );
}

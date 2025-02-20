import { Column } from '@tanstack/react-table';
import { ArrowDown, ArrowUp, ChevronsUpDown } from 'lucide-react';
import { cn } from '@/lib/utils';
import { Button } from '@/components/ui/button';
import { t } from '@lingui/core/macro';
interface DataTableColumnHeaderProps<TData, TValue>
  extends React.HTMLAttributes<HTMLDivElement> {
  column: Column<TData, TValue>;
  title: string;
}

export function DataTableColumnHeader<TData, TValue>({
  column,
  title,
  className,
}: DataTableColumnHeaderProps<TData, TValue>) {
  if (!column.getCanSort()) {
    return <div className={cn(className)}>{title}</div>;
  }

  const isSorted = column.getIsSorted();
  const sortDirection = isSorted === 'desc' ? 'descending' : 'ascending';

  return (
    <div className={cn('flex items-center space-x-2', className)}>
      <Button
        variant='ghost'
        size='sm'
        className='-ml-3 h-8 data-[state=open]:bg-accent'
        onClick={() => column.toggleSorting(column.getIsSorted() !== 'desc')}
        aria-label={
          t`Sort by ` + title + (isSorted ? ` (${sortDirection})` : '')
        }
        aria-sort={isSorted ? sortDirection : 'none'}
      >
        <span>{title}</span>
        {isSorted === 'desc' ? (
          <ArrowDown className='ml-2 h-4 w-4' aria-hidden='true' />
        ) : isSorted === 'asc' ? (
          <ArrowUp className='ml-2 h-4 w-4' aria-hidden='true' />
        ) : (
          <ChevronsUpDown
            className='ml-2 h-4 w-4 opacity-0 group-hover:opacity-100'
            aria-hidden='true'
          />
        )}
      </Button>
    </div>
  );
}

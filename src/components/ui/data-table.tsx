import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
import { Trans } from '@lingui/react/macro';
import { t } from '@lingui/core/macro';
import {
  ColumnDef,
  flexRender,
  getCoreRowModel,
  getSortedRowModel,
  OnChangeFn,
  Row,
  SortingState,
  useReactTable,
} from '@tanstack/react-table';

// Add responsive property to column definition
declare module '@tanstack/react-table' {
  interface ColumnMeta<TData, TValue> {
    className?: string;
  }
}

interface DataTableProps<TData, TValue> {
  columns: ColumnDef<TData, TValue>[];
  data: TData[];
  state?: {
    sorting?: SortingState;
    rowSelection?: Record<string, boolean>;
    maxRows?: number;
  };
  onSortingChange?: OnChangeFn<SortingState>;
  getRowStyles?: (row: Row<TData>) => {
    className?: string;
    onClick?: () => void;
  };
  getRowId?: (originalRow: TData) => string;
  showTotalRows?: boolean;
  rowLabel?: string;
  rowLabelPlural?: string;
}

export function DataTable<TData, TValue>({
  columns,
  data,
  state,
  onSortingChange,
  getRowStyles,
  getRowId,
  showTotalRows = true,
  rowLabel = 'row',
  rowLabelPlural = 'rows',
}: DataTableProps<TData, TValue>) {
  const table = useReactTable({
    data,
    columns,
    state,
    onSortingChange,
    getCoreRowModel: getCoreRowModel(),
    getSortedRowModel: getSortedRowModel(),
    getRowId,
  });

  const length = data.length;
  const showingLabel = state?.maxRows
    ? t`Showing ${length} of ${state.maxRows} ${rowLabelPlural}`
    : t`Showing ${length} ${length !== 1 ? rowLabelPlural : rowLabel}`;

  return (
    <div>
      <div className='rounded-md border'>
        <Table aria-label='Table'>
          <TableHeader>
            {table.getHeaderGroups().map((headerGroup) => (
              <TableRow key={headerGroup.id}>
                {headerGroup.headers.map((header) => (
                  <TableHead
                    key={header.id}
                    role='columnheader'
                    className={`truncate px-2 ${header.column.columnDef.meta?.className ?? ''}`}
                    style={{ width: `${header.getSize()}px` }}
                  >
                    {header.isPlaceholder
                      ? null
                      : flexRender(
                          header.column.columnDef.header,
                          header.getContext(),
                        )}
                  </TableHead>
                ))}
              </TableRow>
            ))}
          </TableHeader>
          <TableBody>
            {table.getRowModel().rows?.length ? (
              table.getRowModel().rows.map((row) => (
                <TableRow
                  key={row.id}
                  data-state={row.getIsSelected() && 'selected'}
                  {...getRowStyles?.(row)}
                >
                  {row.getVisibleCells().map((cell) => (
                    <TableCell
                      key={cell.id}
                      role='cell'
                      className={`truncate px-2 ${cell.column.columnDef.meta?.className ?? ''}`}
                    >
                      {flexRender(
                        cell.column.columnDef.cell,
                        cell.getContext(),
                      )}
                    </TableCell>
                  ))}
                </TableRow>
              ))
            ) : (
              <TableRow>
                <TableCell
                  colSpan={columns.length}
                  className='h-24 text-center'
                  role='cell'
                >
                  <Trans>No results.</Trans>
                </TableCell>
              </TableRow>
            )}
          </TableBody>
        </Table>
      </div>
      {showTotalRows && (
        <div
          className='text-sm text-muted-foreground mt-1 mb-2'
          aria-label={showingLabel}
        >
          {showingLabel}
        </div>
      )}
    </div>
  );
}

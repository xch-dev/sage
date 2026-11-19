import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import {
  Column,
  ColumnDef,
  columnSizingFeature,
  createSortedRowModel,
  flexRender,
  metaHelper,
  OnChangeFn,
  Row,
  RowData,
  RowSelectionState,
  rowSelectionFeature,
  rowSortingFeature,
  sortFn_alphanumeric,
  sortFn_basic,
  sortFn_datetime,
  sortFn_text,
  SortingState,
  Table as TanstackTable,
  tableFeatures,
  useTable,
} from '@tanstack/react-table';

// Every table in the app shares these features. The sort functions are the
// ones `sortFn: 'auto'` picks between, which v9 only resolves when registered.
export const dataTableFeatures = tableFeatures({
  columnSizingFeature,
  rowSelectionFeature,
  rowSortingFeature,
  sortedRowModel: createSortedRowModel(),
  sortFns: {
    alphanumeric: sortFn_alphanumeric,
    basic: sortFn_basic,
    datetime: sortFn_datetime,
    text: sortFn_text,
  },
  columnMeta: metaHelper<{
    className?: string;
    cellClassName?: string;
  }>(),
});

export type DataTableFeatures = typeof dataTableFeatures;
export type DataTableColumnDef<
  TData extends RowData,
  TValue = unknown,
> = ColumnDef<DataTableFeatures, TData, TValue>;
export type DataTableColumn<TData extends RowData, TValue = unknown> = Column<
  DataTableFeatures,
  TData,
  TValue
>;
export type DataTableRow<TData extends RowData> = Row<DataTableFeatures, TData>;
export type DataTableInstance<TData extends RowData> = TanstackTable<
  DataTableFeatures,
  TData
>;

interface DataTableProps<TData extends RowData> {
  columns: DataTableColumnDef<TData>[];
  data: TData[];
  state?: {
    sorting?: SortingState;
    rowSelection?: RowSelectionState;
    maxRows?: number;
  };
  onSortingChange?: OnChangeFn<SortingState>;
  onRowSelectionChange?: OnChangeFn<RowSelectionState>;
  getRowStyles?: (row: DataTableRow<TData>) => {
    className?: string;
    onClick?: () => void;
  };
  getRowId?: (originalRow: TData) => string;
  showTotalRows?: boolean;
  rowLabel?: string;
  rowLabelPlural?: string;
}

export function DataTable<TData extends RowData>({
  columns,
  data,
  state,
  onSortingChange,
  onRowSelectionChange,
  getRowStyles,
  getRowId,
  showTotalRows = true,
  rowLabel = 'row',
  rowLabelPlural = 'rows',
}: DataTableProps<TData>) {
  const table = useTable({
    features: dataTableFeatures,
    data,
    columns,
    state,
    onSortingChange,
    onRowSelectionChange,
    enableRowSelection: true,
    getRowId: getRowId,
  });

  const length = data.length;
  const maxRows = state?.maxRows;
  const rowLabelText = length !== 1 ? rowLabelPlural : rowLabel;
  const showingLabel = maxRows
    ? t`Showing ${length} of ${maxRows} ${rowLabelPlural}`
    : t`Showing ${length} ${rowLabelText}`;

  return (
    <div>
      <div className='border rounded-lg table-theme'>
        <Table aria-label='Table' style={{ border: 'none' }}>
          <TableHeader>
            {table.getHeaderGroups().map((headerGroup) => (
              <TableRow key={headerGroup.id}>
                {headerGroup.headers.map((header) => (
                  <TableHead
                    key={header.id}
                    role='columnheader'
                    className={`${header.column.columnDef.meta?.cellClassName ?? 'truncate px-2'} ${header.column.columnDef.meta?.className ?? ''}`}
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
                  {row.getAllCells().map((cell) => (
                    <TableCell
                      key={cell.id}
                      role='cell'
                      className={`${cell.column.columnDef.meta?.cellClassName ?? 'truncate px-2'} ${cell.column.columnDef.meta?.className ?? ''}`}
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

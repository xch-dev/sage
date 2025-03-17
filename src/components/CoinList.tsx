import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
import { fromMojos, formatTimestamp } from '@/lib/utils';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import {
  ColumnDef,
  flexRender,
  getCoreRowModel,
  getFilteredRowModel,
  getPaginationRowModel,
  getSortedRowModel,
  RowSelectionState,
  SortingState,
  useReactTable,
} from '@tanstack/react-table';
import {
  ArrowDown,
  ArrowUp,
  ChevronLeft,
  ChevronRight,
  FilterIcon,
  FilterXIcon,
} from 'lucide-react';
import { useState } from 'react';
import { CoinRecord } from '../bindings';
import { Button } from './ui/button';
import { Checkbox } from './ui/checkbox';
import { NumberFormat } from './NumberFormat';

export interface CoinListProps {
  precision: number;
  coins: CoinRecord[];
  selectedCoins: RowSelectionState;
  setSelectedCoins: React.Dispatch<React.SetStateAction<RowSelectionState>>;
  actions?: React.ReactNode;
}

export default function CoinList(props: CoinListProps) {
  const [sorting, setSorting] = useState<SortingState>([
    { id: 'created_height', desc: true },
  ]);
  const [showUnspentOnly, setShowUnspentOnly] = useState(false);

  const columns: ColumnDef<CoinRecord>[] = [
    {
      id: 'select',
      size: 20,
      meta: {
        className: 'w-[20px] min-w-[20px] max-w-[20px]',
      },
      header: ({ table }) => (
        <div className='flex'>
          <Checkbox
            className='mx-1'
            checked={
              table.getIsAllPageRowsSelected() ||
              (table.getIsSomePageRowsSelected() && 'indeterminate')
            }
            onCheckedChange={(value) =>
              table.toggleAllPageRowsSelected(!!value)
            }
            aria-label={t`Select all coins`}
          />
        </div>
      ),
      cell: ({ row }) => (
        <div>
          <Checkbox
            className='mx-1'
            checked={row.getIsSelected()}
            onCheckedChange={(value) => row.toggleSelected(!!value)}
            aria-label={t`Select coin row`}
          />
        </div>
      ),
      enableSorting: false,
      enableHiding: false,
    },
    {
      accessorKey: 'coin_id',
      size: 100,
      meta: {
        className: 'w-[100px] min-w-[100px]',
      },
      header: ({ column }) => {
        return (
          <div>
            <Button
              className='px-0'
              variant='link'
              onClick={() =>
                column.toggleSorting(column.getIsSorted() === 'asc')
              }
            >
              <Trans>Coin</Trans>
              {column.getIsSorted() === 'asc' ? (
                <ArrowUp className='ml-2 h-4 w-4' aria-hidden='true' />
              ) : column.getIsSorted() === 'desc' ? (
                <ArrowDown className='ml-2 h-4 w-4' aria-hidden='true' />
              ) : (
                <span className='ml-2 w-4 h-4' />
              )}
            </Button>
          </div>
        );
      },
      cell: ({ row }) => <div className='truncate'>{row.original.coin_id}</div>,
    },
    {
      accessorKey: 'amount',
      size: 80,
      meta: {
        className: 'w-[80px] min-w-[80px]',
      },
      header: ({ column }) => {
        return (
          <div>
            <Button
              className='px-0'
              variant='link'
              onClick={() =>
                column.toggleSorting(column.getIsSorted() === 'asc')
              }
            >
              <Trans>Amount</Trans>
              {column.getIsSorted() === 'asc' ? (
                <ArrowUp className='ml-2 h-4 w-4' aria-hidden='true' />
              ) : column.getIsSorted() === 'desc' ? (
                <ArrowDown className='ml-2 h-4 w-4' aria-hidden='true' />
              ) : (
                <span className='ml-2 w-4 h-4' />
              )}
            </Button>
          </div>
        );
      },
      cell: (info) => (
        <div className='font-mono truncate'>
          <NumberFormat
            value={fromMojos(info.getValue() as string, props.precision)}
            minimumFractionDigits={0}
            maximumFractionDigits={props.precision}
          />
        </div>
      ),
    },
    {
      accessorKey: 'created_height',
      size: 70,
      meta: {
        className: 'w-[70px] min-w-[70px]',
      },
      sortingFn: (rowA, rowB) => {
        const addSpend = 1_000_000_000;
        const addCreate = 2_000_000_000;

        const aPending =
          !!rowA.original.spend_transaction_id && !rowA.original.spent_height;
        const bPending =
          !!rowB.original.spend_transaction_id && !rowB.original.spent_height;

        const a =
          (rowA.original.created_height ?? 0) +
          (aPending
            ? addSpend
            : rowA.original.create_transaction_id
              ? addCreate
              : 0);

        const b =
          (rowB.original.created_height ?? 0) +
          (bPending
            ? addSpend
            : rowB.original.create_transaction_id
              ? addCreate
              : 0);

        return a < b ? -1 : a > b ? 1 : 0;
      },
      header: ({ column }) => {
        return (
          <div>
            <Button
              className='px-0'
              variant='link'
              onClick={() =>
                column.toggleSorting(column.getIsSorted() === 'asc')
              }
            >
              <Trans>Confirmed</Trans>
              {column.getIsSorted() === 'asc' ? (
                <ArrowUp className='ml-2 h-4 w-4' aria-hidden='true' />
              ) : column.getIsSorted() === 'desc' ? (
                <ArrowDown className='ml-2 h-4 w-4' aria-hidden='true' />
              ) : (
                <span className='ml-2 w-4 h-4' />
              )}
            </Button>
          </div>
        );
      },
      cell: ({ row }) => (
        <div className='truncate'>
          {row.original.created_timestamp
            ? formatTimestamp(row.original.created_timestamp, 'short', 'short')
            : row.original.create_transaction_id
              ? t`Pending...`
              : ''}
        </div>
      ),
    },
    {
      accessorKey: 'spent_height',
      size: 70,
      meta: {
        className: 'hidden md:table-cell w-[70px] min-w-[70px]',
      },
      sortingFn: (rowA, rowB) => {
        const a =
          (rowA.original.spent_height ?? 0) +
          (rowA.original.spend_transaction_id ? 10000000 : 0) +
          (rowA.original.offer_id ? 20000000 : 0);
        const b =
          (rowB.original.spent_height ?? 0) +
          (rowB.original.spend_transaction_id ? 10000000 : 0) +
          (rowB.original.offer_id ? 20000000 : 0);
        return a < b ? -1 : a > b ? 1 : 0;
      },
      header: ({ column }) => {
        return (
          <div className='hidden md:table-cell w-[70px] min-w-[70px]'>
            <Button
              className='px-0 mr-2'
              variant='link'
              onClick={() =>
                column.toggleSorting(column.getIsSorted() === 'asc')
              }
            >
              <Trans>Spent</Trans>
              {column.getIsSorted() === 'asc' ? (
                <ArrowUp className='ml-2 h-4 w-4' aria-hidden='true' />
              ) : column.getIsSorted() === 'desc' ? (
                <ArrowDown className='ml-2 h-4 w-4' aria-hidden='true' />
              ) : (
                <span className='ml-2 w-4 h-4' />
              )}
            </Button>
            <Button
              size='icon'
              variant='ghost'
              className='text-foreground'
              onClick={() => {
                setShowUnspentOnly(!showUnspentOnly);
                column.setFilterValue(showUnspentOnly ? t`Unspent` : '');

                if (!showUnspentOnly) {
                  setSorting([{ id: 'spent_height', desc: true }]);
                } else {
                  setSorting([{ id: 'created_height', desc: true }]);
                }
              }}
              aria-label={
                showUnspentOnly ? t`Show all coins` : t`Show unspent coins only`
              }
            >
              {showUnspentOnly ? (
                <FilterIcon className='h-4 w-4' aria-hidden='true' />
              ) : (
                <FilterXIcon className='h-4 w-4' aria-hidden='true' />
              )}
            </Button>
          </div>
        );
      },
      filterFn: (row, _, filterValue) => {
        return (
          filterValue === t`Unspent` &&
          !row.original.spend_transaction_id &&
          !row.original.spent_height &&
          !row.original.offer_id
        );
      },
      cell: ({ row }) => (
        <div className='truncate'>
          {row.original.spent_timestamp
            ? formatTimestamp(row.original.spent_timestamp, 'short', 'short')
            : (row.original.spent_height ??
              (row.original.spend_transaction_id
                ? t`Pending...`
                : row.original.offer_id
                  ? t`Offered...`
                  : ''))}
        </div>
      ),
    },
  ];

  const table = useReactTable({
    data: props.coins,
    columns,
    columnResizeMode: 'onChange',
    getCoreRowModel: getCoreRowModel(),
    getSortedRowModel: getSortedRowModel(),
    getPaginationRowModel: getPaginationRowModel(),
    getFilteredRowModel: getFilteredRowModel(),
    onSortingChange: setSorting,
    state: {
      sorting,
      rowSelection: props.selectedCoins,
    },
    getRowId: (row) => row.coin_id,
    onRowSelectionChange: props.setSelectedCoins,
    initialState: {
      pagination: {
        pageSize: 10,
      },
      columnFilters: [
        {
          id: 'spent_timestamp',
          value: t`Unspent`,
        },
      ],
    },
  });

  return (
    <div>
      <div className='rounded-md border overflow-x-auto'>
        <Table>
          <TableHeader>
            {table.getHeaderGroups().map((headerGroup) => (
              <TableRow key={headerGroup.id}>
                {headerGroup.headers.map((header) => (
                  <TableHead
                    key={header.id}
                    className='whitespace-nowrap'
                    style={{ width: header.column.getSize() }}
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
                  onClick={() => row.toggleSelected(!row.getIsSelected())}
                >
                  {row.getVisibleCells().map((cell) => (
                    <TableCell
                      key={cell.id}
                      className='whitespace-nowrap'
                      style={{ width: cell.column.getSize() }}
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
                >
                  <Trans>No results.</Trans>
                </TableCell>
              </TableRow>
            )}
          </TableBody>
        </Table>
      </div>
      <div className='pt-4'>
        <div className='flex items-center justify-between'>
          <div className='flex space-x-2'>{props.actions}</div>
          <div className='flex space-x-2'>
            <Button
              variant='outline'
              size='icon'
              onClick={() => table.previousPage()}
              disabled={!table.getCanPreviousPage()}
              aria-label={t`Previous page`}
            >
              <ChevronLeft className='h-4 w-4' aria-hidden='true' />
            </Button>
            <Button
              variant='outline'
              size='icon'
              onClick={() => table.nextPage()}
              disabled={!table.getCanNextPage()}
              aria-label={t`Next page`}
            >
              <ChevronRight className='h-4 w-4' aria-hidden='true' />
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
}

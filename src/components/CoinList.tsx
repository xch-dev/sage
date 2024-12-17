import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
import { toDecimal } from '@/lib/utils';
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
      header: ({ table }) => (
        <Checkbox
          className='mx-2'
          checked={
            table.getIsAllPageRowsSelected() ||
            (table.getIsSomePageRowsSelected() && 'indeterminate')
          }
          onCheckedChange={(value) => table.toggleAllPageRowsSelected(!!value)}
          aria-label='Select all'
        />
      ),
      cell: ({ row }) => (
        <Checkbox
          className='mx-2'
          checked={row.getIsSelected()}
          onCheckedChange={(value) => row.toggleSelected(!!value)}
          aria-label='Select row'
        />
      ),
      enableSorting: false,
      enableHiding: false,
    },
    {
      accessorKey: 'coin_id',
      header: ({ column }) => {
        return (
          <Button
            className='px-0'
            variant='link'
            onClick={() => column.toggleSorting(column.getIsSorted() === 'asc')}
          >
            Coin
            {column.getIsSorted() === 'asc' ? (
              <ArrowUp className='ml-2 h-4 w-4' />
            ) : column.getIsSorted() === 'desc' ? (
              <ArrowDown className='ml-2 h-4 w-4' />
            ) : (
              <span className='ml-2 w-4 h-4' />
            )}
          </Button>
        );
      },
      size: 100,
      cell: ({ row }) => (
        <div className='truncate overflow-hidden'>{row.original.coin_id}</div>
      ),
    },
    {
      accessorKey: 'amount',
      header: ({ column }) => {
        return (
          <Button
            className='px-0'
            variant='link'
            onClick={() => column.toggleSorting(column.getIsSorted() === 'asc')}
          >
            Amount
            {column.getIsSorted() === 'asc' ? (
              <ArrowUp className='ml-2 h-4 w-4' />
            ) : column.getIsSorted() === 'desc' ? (
              <ArrowDown className='ml-2 h-4 w-4' />
            ) : (
              <span className='ml-2 w-4 h-4' />
            )}
          </Button>
        );
      },
      cell: (info) => (
        <span className='font-mono'>
          {toDecimal(info.getValue() as string, props.precision)}
        </span>
      ),
    },
    {
      accessorKey: 'created_height',
      sortingFn: (rowA, rowB) => {
        const addSpend = 1_000_000_000;
        const addCreate = 2_000_000_000;

        const a =
          (rowA.original.created_height ?? 0) +
          (rowA.original.spend_transaction_id
            ? addSpend
            : rowA.original.create_transaction_id
              ? addCreate
              : 0);

        const b =
          (rowB.original.created_height ?? 0) +
          (rowB.original.spend_transaction_id
            ? addSpend
            : rowB.original.create_transaction_id
              ? addCreate
              : 0);

        return a < b ? -1 : a > b ? 1 : 0;
      },
      header: ({ column }) => {
        return (
          <Button
            className='px-0'
            variant='link'
            onClick={() => column.toggleSorting(column.getIsSorted() === 'asc')}
          >
            Confirmed
            {column.getIsSorted() === 'asc' ? (
              <ArrowUp className='ml-2 h-4 w-4' />
            ) : column.getIsSorted() === 'desc' ? (
              <ArrowDown className='ml-2 h-4 w-4' />
            ) : (
              <span className='ml-2 w-4 h-4' />
            )}
          </Button>
        );
      },
      cell: ({ row }) => (
        <div className='truncate overflow-hidden'>
          {row.original.created_height ??
            (row.original.create_transaction_id ? 'Pending...' : '')}
        </div>
      ),
    },
    {
      accessorKey: 'spent_height',
      sortingFn: (rowA, rowB) => {
        const a =
          (rowA.original.spent_height ?? 0) +
          (rowA.original.spend_transaction_id ? 10000000 : 0);
        const b =
          (rowB.original.spent_height ?? 0) +
          (rowB.original.spend_transaction_id ? 10000000 : 0);
        return a < b ? -1 : a > b ? 1 : 0;
      },
      header: ({ column }) => {
        return (
          <div className='flex items-center'>
            <Button
              className='px-0 mr-2'
              variant='link'
              onClick={() =>
                column.toggleSorting(column.getIsSorted() === 'asc')
              }
            >
              Spent
              {column.getIsSorted() === 'asc' ? (
                <ArrowUp className='ml-2 h-4 w-4' />
              ) : column.getIsSorted() === 'desc' ? (
                <ArrowDown className='ml-2 h-4 w-4' />
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
                column.setFilterValue(showUnspentOnly ? 'Unspent' : '');
              }}
            >
              {showUnspentOnly ? (
                <FilterIcon className='h-4 w-4' />
              ) : (
                <FilterXIcon className='h-4 w-4' />
              )}
            </Button>
          </div>
        );
      },
      filterFn: (row, _, filterValue) => {
        return (
          filterValue === 'Unspent' &&
          !row.original.spend_transaction_id &&
          !row.original.spent_height
        );
      },
      cell: ({ row }) => (
        <div className='truncate overflow-hidden'>
          {row.original.spent_height ??
            (row.original.spend_transaction_id ? 'Pending...' : '')}
        </div>
      ),
    },
  ];

  const table = useReactTable({
    data: props.coins,
    columns,
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
          id: 'spent_height',
          value: 'Unspent',
        },
      ],
    },
  });

  return (
    <div>
      <div className='rounded-md border'>
        <Table>
          <TableHeader>
            {table.getHeaderGroups().map((headerGroup) => (
              <TableRow key={headerGroup.id}>
                {headerGroup.headers.map((header) => (
                  <TableHead key={header.id}>
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
                      style={{
                        maxWidth: cell.column.columnDef.size,
                      }}
                      className={
                        'h-12' +
                        (row.original.spend_transaction_id ||
                        row.original.create_transaction_id
                          ? ' pulsate-opacity'
                          : '')
                      }
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
                  No results.
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
            >
              <ChevronLeft className='h-4 w-4' />
            </Button>
            <Button
              variant='outline'
              size='icon'
              onClick={() => table.nextPage()}
              disabled={!table.getCanNextPage()}
            >
              <ChevronRight className='h-4 w-4' />
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
}

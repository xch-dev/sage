import { formatTimestamp, fromMojos } from '@/lib/utils';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import {
  ColumnDef,
  Row,
  RowSelectionState,
  SortingState,
} from '@tanstack/react-table';
import { openUrl } from '@tauri-apps/plugin-opener';
import { ArrowDown, ArrowUp, FilterIcon, FilterXIcon } from 'lucide-react';
import { useState } from 'react';
import { CoinRecord } from '../bindings';
import { NumberFormat } from './NumberFormat';
import { SimplePagination } from './SimplePagination';
import { Button } from './ui/button';
import { Checkbox } from './ui/checkbox';
import { DataTable } from './ui/data-table';

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
  const [showSpentCoins, setShowSpentCoins] = useState(false);
  const [currentPage, setCurrentPage] = useState(0);
  const pageSize = 10;

  const filteredCoins = !showSpentCoins
    ? props.coins.filter(
        (coin) =>
          !coin.spend_transaction_id && !coin.spent_height && !coin.offer_id,
      )
    : props.coins;

  // Column definitions
  const columns: ColumnDef<CoinRecord>[] = [
    {
      id: 'select',
      size: 30,
      header: ({ table }) => (
        <div className='flex justify-center items-center'>
          <Checkbox
            checked={
              table.getIsAllPageRowsSelected() ||
              (table.getIsSomePageRowsSelected() && 'indeterminate')
            }
            onCheckedChange={(value) => {
              table.toggleAllPageRowsSelected(!!value);

              // Update the external selection state
              const newSelections = { ...props.selectedCoins };

              // Get all visible rows on the current page
              const pageRows = table.getRowModel().rows;

              pageRows.forEach((row) => {
                const rowId = row.original.coin_id;
                newSelections[rowId] = !!value;
              });

              props.setSelectedCoins(newSelections);
            }}
            aria-label={t`Select all coins`}
          />
        </div>
      ),
      cell: ({ row }) => (
        <div className='flex justify-center items-center'>
          <Checkbox
            checked={row.getIsSelected()}
            onCheckedChange={(value) => {
              row.toggleSelected(!!value);

              // Update the external selection state
              const rowId = row.original.coin_id;
              props.setSelectedCoins((prev) => ({
                ...prev,
                [rowId]: !!value,
              }));
            }}
            aria-label={t`Select coin row`}
          />
        </div>
      ),
      enableSorting: false,
    },
    {
      accessorKey: 'coin_id',
      size: 100,
      header: ({ column }) => (
        <Button
          className='px-0'
          variant='link'
          onClick={() => column.toggleSorting(column.getIsSorted() === 'asc')}
        >
          <Trans>Coin ID</Trans>
          {column.getIsSorted() === 'asc' ? (
            <ArrowUp className='ml-2 h-4 w-4' aria-hidden='true' />
          ) : column.getIsSorted() === 'desc' ? (
            <ArrowDown className='ml-2 h-4 w-4' aria-hidden='true' />
          ) : (
            <span className='ml-2 w-4 h-4' />
          )}
        </Button>
      ),
      cell: ({ row }) => {
        const coinId = row.original.coin_id;
        return (
          <div
            className='cursor-pointer truncate hover:underline'
            onClick={(e) => {
              e.stopPropagation();
              openUrl(`https://spacescan.io/coin/0x${coinId}`);
            }}
            aria-label={t`View coin ${coinId} on Spacescan.io`}
            role='button'
            onKeyDown={(e) => {
              if (e.key === 'Enter' || e.key === ' ') {
                e.stopPropagation();
                openUrl(`https://spacescan.io/coin/0x${coinId}`);
              }
            }}
          >
            {coinId}
          </div>
        );
      },
    },
    {
      accessorKey: 'amount',
      size: 100,
      header: ({ column }) => (
        <Button
          className='px-0'
          variant='link'
          onClick={() => column.toggleSorting(column.getIsSorted() === 'asc')}
        >
          <span className='text-foreground hover:underline'>
            <Trans>Amount</Trans>
          </span>
          {column.getIsSorted() === 'asc' ? (
            <ArrowUp className='ml-2 h-4 w-4' aria-hidden='true' />
          ) : column.getIsSorted() === 'desc' ? (
            <ArrowDown className='ml-2 h-4 w-4' aria-hidden='true' />
          ) : (
            <span className='ml-2 w-4 h-4' />
          )}
        </Button>
      ),
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
      header: ({ column }) => (
        <Button
          className='px-0'
          variant='link'
          onClick={() => column.toggleSorting(column.getIsSorted() === 'asc')}
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
      ),
      size: 140,
      cell: ({ row }) =>
        row.original.created_timestamp
          ? formatTimestamp(row.original.created_timestamp, 'short', 'short')
          : row.original.created_height
            ? row.original.created_height.toString()
            : row.original.create_transaction_id
              ? t`Pending...`
              : '',
    },
    {
      accessorKey: 'spent_height',
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
      header: ({ column }) => (
        <div className='flex items-center space-x-1'>
          <Button
            className='px-0'
            variant='link'
            onClick={() => column.toggleSorting(column.getIsSorted() === 'asc')}
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
            className='h-6 w-6 p-0 ml-1'
            onClick={() => {
              const newShowSpentCoins = !showSpentCoins;
              setShowSpentCoins(newShowSpentCoins);
              if (newShowSpentCoins) {
                setSorting([{ id: 'spent_height', desc: true }]);
              } else {
                setSorting([{ id: 'created_height', desc: true }]);
              }
              setCurrentPage(0); // Reset to first page on filter change
            }}
            aria-label={
              showSpentCoins ? t`Show all coins` : t`Show unspent coins only`
            }
          >
            {showSpentCoins ? (
              <FilterIcon className='h-3 w-3' aria-hidden='true' />
            ) : (
              <FilterXIcon className='h-3 w-3' aria-hidden='true' />
            )}
          </Button>
        </div>
      ),
      size: 140,
      cell: ({ row }) =>
        row.original.spent_timestamp
          ? formatTimestamp(row.original.spent_timestamp, 'short', 'short')
          : (row.original.spent_height ??
            (row.original.spend_transaction_id
              ? t`Pending...`
              : row.original.offer_id
                ? t`Offered...`
                : '')),
    },
  ];

  const getRowStyles = (row: Row<CoinRecord>) => {
    return {
      className: row.getIsSelected() ? 'bg-primary/10' : '',
      onClick: () => {
        const newValue = !row.getIsSelected();
        row.toggleSelected(newValue);

        // Update the external selection state
        const rowId = row.original.coin_id;
        props.setSelectedCoins((prev) => ({
          ...prev,
          [rowId]: newValue,
        }));
      },
    };
  };

  // Custom sort function to sort the entire dataset
  const sortData = (data: CoinRecord[], sortingState: SortingState) => {
    if (!sortingState.length) return data;

    return [...data].sort((a, b) => {
      for (const sort of sortingState) {
        const column = columns.find(
          (col) => (col as any).accessorKey === sort.id || col.id === sort.id,
        );
        if (!column) continue;

        let result = 0;

        if (column.sortingFn && typeof column.sortingFn === 'function') {
          // Use the column's custom sorting function
          const rowA = { original: a };
          const rowB = { original: b };
          result = (column.sortingFn as any)(rowA, rowB, sort.id);
        } else {
          // Default sorting based on accessor key
          const aValue = (a as any)[sort.id];
          const bValue = (b as any)[sort.id];

          if (aValue === undefined && bValue === undefined) result = 0;
          else if (aValue === undefined) result = 1;
          else if (bValue === undefined) result = -1;
          else result = aValue < bValue ? -1 : aValue > bValue ? 1 : 0;
        }

        if (result !== 0) return sort.desc ? -result : result;
      }
      return 0;
    });
  };

  // Sort the entire dataset first, then paginate
  const sortedCoins = sortData(filteredCoins, sorting);

  // Calculate pagination after sorting
  const pageCount = Math.ceil(sortedCoins.length / pageSize);
  const paginatedCoins = sortedCoins.slice(
    currentPage * pageSize,
    (currentPage + 1) * pageSize,
  );

  return (
    <div>
      <DataTable
        columns={columns}
        data={paginatedCoins}
        state={{
          sorting,
          rowSelection: props.selectedCoins,
        }}
        onSortingChange={setSorting}
        getRowStyles={getRowStyles}
        getRowId={(row) => row.coin_id}
      />
      <div className='flex-shrink-0 py-4'>
        <SimplePagination
          currentPage={currentPage}
          pageCount={pageCount}
          setCurrentPage={setCurrentPage}
          size='sm'
          align='between'
          actions={props.actions}
        />
      </div>
    </div>
  );
}

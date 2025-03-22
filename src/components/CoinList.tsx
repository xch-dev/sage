import { fromMojos, formatTimestamp } from '@/lib/utils';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import {
  ColumnDef,
  Row,
  RowSelectionState,
  SortingState,
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
import { openUrl } from '@tauri-apps/plugin-opener';
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

  const filteredCoins = showSpentCoins
    ? props.coins.filter(
        (coin) =>
          !coin.spend_transaction_id && !coin.spent_height && !coin.offer_id,
      )
    : props.coins;

  // Column definitions
  const columns: ColumnDef<CoinRecord>[] = [
    {
      id: 'select',
      meta: {
        className: 'w-[30px] max-w-[30px]',
      },
      header: ({ table }) => (
        <div className='pl-1'>
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
        <div className='pl-2 md:pl-1'>
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
      meta: {
        className: 'w-[70px] min-w-[70px] md:min-w-[100px]',
      },
      header: ({ column }) => (
        <div>
          <Button
            className='px-0'
            variant='link'
            onClick={() => column.toggleSorting(column.getIsSorted() === 'asc')}
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
      ),
      cell: ({ row }) => (
        <div
          className='cursor-pointer truncate hover:underline'
          onClick={(e) => {
            e.stopPropagation();
            openUrl(`https://spacescan.io/coin/0x${row.original.coin_id}`);
          }}
          aria-label={t`View coin ${row.original.coin_id} on Spacescan.io`}
          role='button'
          onKeyDown={(e) => {
            if (e.key === 'Enter' || e.key === ' ') {
              e.stopPropagation();
              openUrl(`https://spacescan.io/coin/0x${row.original.coin_id}`);
            }
          }}
        >
          {row.original.coin_id}
        </div>
      ),
    },
    {
      accessorKey: 'amount',
      meta: {
        className:
          'text-right w-[60px] md:w-[80px] min-w-[60px] md:min-w-[80px]',
      },
      header: ({ column }) => (
        <div
          className='text-right cursor-pointer'
          onClick={() => column.toggleSorting(column.getIsSorted() === 'asc')}
        >
          {column.getIsSorted() === 'asc' ? (
            <ArrowUp className='mr-2 h-4 w-4 inline-block' aria-hidden='true' />
          ) : column.getIsSorted() === 'desc' ? (
            <ArrowDown
              className='mr-2 h-4 w-4 inline-block'
              aria-hidden='true'
            />
          ) : (
            <span className='mr-2 w-4 h-4 inline-block' />
          )}
          <span className='text-foreground hover:underline'>
            <Trans>Amount</Trans>
          </span>
        </div>
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
      meta: {
        className: 'hidden md:table-cell w-[70px] min-w-[70px]',
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
      header: ({ column }) => (
        <div className='hidden md:block'>
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
        </div>
      ),
      cell: ({ row }) => (
        <div className='hidden md:block truncate'>
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
      header: ({ column }) => (
        <div className='hidden md:block'>
          <div className='flex items-center space-x-1'>
            <Button
              className='px-0'
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
              className='h-6 w-6 p-0 ml-1'
              onClick={() => {
                setShowSpentCoins(!showSpentCoins);
                console.log('showSpentCoins', showSpentCoins);
                if (showSpentCoins) {
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
                <FilterXIcon className='h-3 w-3' aria-hidden='true' />
              ) : (
                <FilterIcon className='h-3 w-3' aria-hidden='true' />
              )}
            </Button>
          </div>
        </div>
      ),
      cell: ({ row }) => (
        <div className='hidden md:block truncate'>
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
      <div className='pt-4'>
        <div className='flex items-center justify-between'>
          <div className='flex space-x-2'>{props.actions}</div>
          <div className='flex space-x-2'>
            <Button
              variant='outline'
              size='icon'
              onClick={() => setCurrentPage(Math.max(0, currentPage - 1))}
              disabled={currentPage === 0}
              aria-label={t`Previous page`}
            >
              <ChevronLeft className='h-4 w-4' aria-hidden='true' />
            </Button>
            <Button
              variant='outline'
              size='icon'
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
      </div>
    </div>
  );
}

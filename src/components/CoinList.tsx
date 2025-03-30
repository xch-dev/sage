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
  currentPage: number;
  totalPages: number;
  setCurrentPage: (page: number) => void;
  maxRows: number;
  actions?: React.ReactNode;
}

export default function CoinList(props: CoinListProps) {
  const [sorting, setSorting] = useState<SortingState>([
    { id: 'created_height', desc: true },
  ]);
  const [showSpentCoins, setShowSpentCoins] = useState(false);

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
            variant='ghost'
            size='icon'
            className='ml-2 h-4 w-4'
            onClick={() => setShowSpentCoins(!showSpentCoins)}
            aria-label={
              showSpentCoins ? t`Hide spent coins` : t`Show spent coins`
            }
          >
            {showSpentCoins ? (
              <FilterXIcon className='h-3 w-3' aria-hidden='true' />
            ) : (
              <FilterIcon className='h-3 w-3' aria-hidden='true' />
            )}
          </Button>
        </div>
      ),
      size: 120,
      cell: ({ row }) =>
        row.original.spent_timestamp
          ? formatTimestamp(row.original.spent_timestamp, 'short', 'short')
          : row.original.spent_height
            ? row.original.spent_height.toString()
            : row.original.spend_transaction_id
              ? t`Pending...`
              : row.original.offer_id
                ? t`Locked in offer`
                : '',
    },
  ];

  const getRowStyles = (row: Row<CoinRecord>) => {
    const coin = row.original;
    const isSpent = !!coin.spent_height || !!coin.spend_transaction_id;
    const isPending = !coin.created_height;

    let className = '';

    if (isSpent) {
      className = 'opacity-50 relative';
    } else if (isPending) {
      className = 'font-medium';
    }

    return {
      className,
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

  const sortData = (data: CoinRecord[], sortingState: SortingState) => {
    const sorted = [...data];

    sorted.sort((a, b) => {
      for (let i = 0; i < sortingState.length; i++) {
        const sort = sortingState[i];
        const { id, desc } = sort;

        if (id === 'created_height') {
          const aPending = !!a.spend_transaction_id && !a.spent_height;
          const bPending = !!b.spend_transaction_id && !b.spent_height;

          const addSpend = 1_000_000_000;
          const addCreate = 2_000_000_000;

          const aVal =
            (a.created_height ?? 0) +
            (aPending ? addSpend : a.create_transaction_id ? addCreate : 0);

          const bVal =
            (b.created_height ?? 0) +
            (bPending ? addSpend : b.create_transaction_id ? addCreate : 0);

          if (aVal !== bVal) {
            return desc ? bVal - aVal : aVal - bVal;
          }
        } else if (id === 'spent_height') {
          const aVal =
            (a.spent_height ?? 0) +
            (a.spend_transaction_id ? 10000000 : 0) +
            (a.offer_id ? 20000000 : 0);
          const bVal =
            (b.spent_height ?? 0) +
            (b.spend_transaction_id ? 10000000 : 0) +
            (b.offer_id ? 20000000 : 0);

          if (aVal !== bVal) {
            return desc ? bVal - aVal : aVal - bVal;
          }
        } else if (id === 'amount') {
          const aVal = BigInt(a.amount);
          const bVal = BigInt(b.amount);

          if (aVal !== bVal) {
            return desc ? (bVal > aVal ? 1 : -1) : aVal > bVal ? 1 : -1;
          }
        } else {
          // Default string comparison for other columns
          const aVal = a[id as keyof CoinRecord] as string | null;
          const bVal = b[id as keyof CoinRecord] as string | null;

          if (aVal !== bVal) {
            if (aVal === null) return desc ? -1 : 1;
            if (bVal === null) return desc ? 1 : -1;
            return desc ? bVal.localeCompare(aVal) : aVal.localeCompare(bVal);
          }
        }
      }

      return 0;
    });

    return sorted;
  };

  const sortedData = sortData(filteredCoins, sorting);

  return (
    <div>
      <DataTable
        data={sortedData}
        columns={columns}
        getRowStyles={getRowStyles}
        onSortingChange={setSorting}
        state={{
          rowSelection: props.selectedCoins,
          sorting,
          maxRows: props.maxRows,
        }}
      />
      <div className='flex-shrink-0 py-4'>
        <SimplePagination
          currentPage={props.currentPage}
          pageCount={props.totalPages}
          setCurrentPage={props.setCurrentPage}
          size='sm'
          align='between'
          actions={props.actions}
        />
      </div>
    </div>
  );
}

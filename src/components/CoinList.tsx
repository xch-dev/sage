import { formatTimestamp, fromMojos } from '@/lib/utils';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import {
  ColumnDef,
  OnChangeFn,
  Row,
  RowSelectionState,
} from '@tanstack/react-table';
import { openUrl } from '@tauri-apps/plugin-opener';
import { ArrowDown, ArrowUp, FilterIcon, FilterXIcon } from 'lucide-react';
import { CoinRecord, CoinSortMode } from '../bindings';
import { NumberFormat } from './NumberFormat';
import { SimplePagination } from './SimplePagination';
import { Button } from './ui/button';
import { Checkbox } from './ui/checkbox';
import { DataTable } from './ui/data-table';
import { useMemo } from 'react';

export interface CoinListProps {
  precision: number;
  coins: CoinRecord[];
  selectedCoins: RowSelectionState;
  setSelectedCoins: React.Dispatch<React.SetStateAction<RowSelectionState>>;
  onRowSelectionChange?: OnChangeFn<RowSelectionState>;
  currentPage: number;
  totalPages: number;
  setCurrentPage: (page: number) => void;
  maxRows: number;
  actions?: React.ReactNode;
  // Backend sorting and filtering props
  sortMode: CoinSortMode;
  sortDirection: boolean; // true = ascending, false = descending
  includeSpentCoins: boolean;
  onSortModeChange: (mode: CoinSortMode) => void;
  onSortDirectionChange: (ascending: boolean) => void;
  onIncludeSpentCoinsChange: (include: boolean) => void;
}

export default function CoinList(props: CoinListProps) {
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
            onCheckedChange={(value) =>
              table.toggleAllPageRowsSelected(!!value)
            }
            aria-label={t`Select all coins`}
          />
        </div>
      ),
      cell: ({ row }) => (
        <div className='flex justify-center items-center'>
          <Checkbox
            checked={row.getIsSelected()}
            onCheckedChange={(value) => row.toggleSelected(!!value)}
            aria-label={t`Select coin row`}
          />
        </div>
      ),
      enableSorting: false,
    },
    {
      accessorKey: 'coin_id',
      size: 100,
      header: () => (
        <Button
          className='px-0'
          variant='link'
          onClick={() => {
            if (props.sortMode === 'coin_id') {
              // Toggle direction only if already sorting by this column
              props.onSortDirectionChange(!props.sortDirection);
            } else {
              // Set column as sort field with default direction (descending)
              props.onSortModeChange('coin_id');
              props.onSortDirectionChange(false);
            }
            props.setCurrentPage(0);
          }}
        >
          <Trans>Coin ID</Trans>
          {props.sortMode === 'coin_id' ? (
            props.sortDirection ? (
              <ArrowUp className='ml-2 h-4 w-4' aria-hidden='true' />
            ) : (
              <ArrowDown className='ml-2 h-4 w-4' aria-hidden='true' />
            )
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
      header: () => (
        <Button
          className='px-0'
          variant='link'
          onClick={() => {
            if (props.sortMode === 'amount') {
              // Toggle direction only if already sorting by this column
              props.onSortDirectionChange(!props.sortDirection);
            } else {
              // Set column as sort field with default direction (descending)
              props.onSortModeChange('amount');
              props.onSortDirectionChange(false);
            }
            props.setCurrentPage(0);
          }}
        >
          <span className='text-foreground hover:underline'>
            <Trans>Amount</Trans>
          </span>
          {props.sortMode === 'amount' ? (
            props.sortDirection ? (
              <ArrowUp className='ml-2 h-4 w-4' aria-hidden='true' />
            ) : (
              <ArrowDown className='ml-2 h-4 w-4' aria-hidden='true' />
            )
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
      header: () => (
        <Button
          className='px-0'
          variant='link'
          onClick={() => {
            if (props.sortMode === 'created_height') {
              // Toggle direction only if already sorting by this column
              props.onSortDirectionChange(!props.sortDirection);
            } else {
              // Set column as sort field with default direction (descending)
              props.onSortModeChange('created_height');
              props.onSortDirectionChange(false);
            }
            props.setCurrentPage(0);
          }}
        >
          <Trans>Confirmed</Trans>
          {props.sortMode === 'created_height' ? (
            props.sortDirection ? (
              <ArrowUp className='ml-2 h-4 w-4' aria-hidden='true' />
            ) : (
              <ArrowDown className='ml-2 h-4 w-4' aria-hidden='true' />
            )
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
      header: () => (
        <div className='flex items-center space-x-1'>
          <Button
            className='px-0'
            variant='link'
            onClick={() => {
              if (props.sortMode === 'spent_height') {
                // Toggle direction only if already sorting by this column
                props.onSortDirectionChange(!props.sortDirection);
              } else {
                // Set column as sort field with default direction (descending)
                props.onSortModeChange('spent_height');
                props.onSortDirectionChange(false);
              }
              props.setCurrentPage(0);
            }}
          >
            <Trans>Spent</Trans>
            {props.sortMode === 'spent_height' ? (
              props.sortDirection ? (
                <ArrowUp className='ml-2 h-4 w-4' aria-hidden='true' />
              ) : (
                <ArrowDown className='ml-2 h-4 w-4' aria-hidden='true' />
              )
            ) : (
              <span className='ml-2 w-4 h-4' />
            )}
          </Button>
          <Button
            variant='ghost'
            size='icon'
            className='ml-2 h-4 w-4'
            onClick={() => {
              const newIncludeSpentCoins = !props.includeSpentCoins;
              props.onIncludeSpentCoinsChange(newIncludeSpentCoins);
              if (newIncludeSpentCoins) {
                props.onSortModeChange('spent_height');
                props.onSortDirectionChange(false);
              } else {
                props.onSortModeChange('created_height');
                props.onSortDirectionChange(false);
              }
              props.setCurrentPage(0);
            }}
            aria-label={
              props.includeSpentCoins
                ? t`Hide spent coins`
                : t`Show spent coins`
            }
          >
            {props.includeSpentCoins ? (
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
    const isSelected = row.getIsSelected();

    let className = '';

    if (isSelected) {
      className = 'bg-accent';
    }

    if (isSpent) {
      className += (className ? ' ' : '') + 'opacity-50 relative';
    } else if (isPending) {
      className += (className ? ' ' : '') + 'font-medium';
    }

    return {
      className,
      onClick: () => row.toggleSelected(!row.getIsSelected()),
    };
  };

  // This function ensures row selection is preserved across page changes
  const getRowId = (coin: CoinRecord) => coin.coin_id;

  // Function to handle row selection changes
  const handleRowSelectionChange: OnChangeFn<RowSelectionState> = (
    updaterOrValue,
  ) => {
    if (typeof updaterOrValue === 'function') {
      const updater = updaterOrValue as (
        prev: RowSelectionState,
      ) => RowSelectionState;
      props.setSelectedCoins((prev) => updater(prev));
    } else {
      props.setSelectedCoins(updaterOrValue);
    }
  };

  // Prepare selected state for current page
  const syncSelectionWithCurrentPage = useMemo(() => {
    const currentPageSelection: RowSelectionState = {};

    // Mark rows on the current page as selected based on global selection state
    props.coins.forEach((coin) => {
      if (props.selectedCoins[coin.coin_id]) {
        currentPageSelection[coin.coin_id] = true;
      }
    });

    return currentPageSelection;
  }, [props.coins, props.selectedCoins]);

  return (
    <div>
      <DataTable
        data={props.coins}
        columns={columns}
        getRowStyles={getRowStyles}
        getRowId={getRowId}
        state={{
          rowSelection: syncSelectionWithCurrentPage,
          maxRows: props.maxRows,
        }}
        onRowSelectionChange={handleRowSelectionChange}
        rowLabel={t`coin`}
        rowLabelPlural={t`coins`}
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

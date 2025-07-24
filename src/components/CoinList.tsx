import { formatTimestamp, fromMojos } from '@/lib/utils';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import {
  ColumnDef,
  OnChangeFn,
  Row,
  RowSelectionState,
  Table,
} from '@tanstack/react-table';
import { openUrl } from '@tauri-apps/plugin-opener';
import { ArrowDown, ArrowUp, FilterIcon, FilterXIcon } from 'lucide-react';
import { useMemo } from 'react';
import { CoinRecord, CoinSortMode } from '../bindings';
import { NumberFormat } from './NumberFormat';
import { SimplePagination } from './SimplePagination';
import { Button } from './ui/button';
import { Checkbox } from './ui/checkbox';
import { DataTable } from './ui/data-table';

// Extract column header and cell components
const SelectAllHeader = ({ table }: { table: Table<CoinRecord> }) => (
  <div className='flex justify-center items-center'>
    <Checkbox
      checked={
        table.getIsAllPageRowsSelected() ||
        (table.getIsSomePageRowsSelected() && 'indeterminate')
      }
      onCheckedChange={(value) => table.toggleAllPageRowsSelected(!!value)}
      aria-label={t`Select all coins`}
    />
  </div>
);

const SelectRowCell = ({ row }: { row: Row<CoinRecord> }) => (
  <div className='flex justify-center items-center'>
    <Checkbox
      checked={row.getIsSelected()}
      onCheckedChange={(value) => row.toggleSelected(!!value)}
      aria-label={t`Select coin row`}
    />
  </div>
);

const CoinIdHeader = ({
  sortMode,
  sortDirection,
  onSortModeChange,
  onSortDirectionChange,
  setCurrentPage,
}: {
  sortMode: CoinSortMode;
  sortDirection: boolean;
  onSortModeChange: (mode: CoinSortMode) => void;
  onSortDirectionChange: (ascending: boolean) => void;
  setCurrentPage: (page: number) => void;
}) => (
  <Button
    className='px-0'
    variant='link'
    onClick={() => {
      if (sortMode === 'coin_id') {
        onSortDirectionChange(!sortDirection);
      } else {
        onSortModeChange('coin_id');
        onSortDirectionChange(false);
      }
      setCurrentPage(0);
    }}
  >
    <Trans>Coin ID</Trans>
    {sortMode === 'coin_id' ? (
      sortDirection ? (
        <ArrowUp className='ml-2 h-4 w-4' aria-hidden='true' />
      ) : (
        <ArrowDown className='ml-2 h-4 w-4' aria-hidden='true' />
      )
    ) : (
      <span className='ml-2 w-4 h-4' />
    )}
  </Button>
);

const CoinIdCell = ({ row }: { row: Row<CoinRecord> }) => {
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
};

const AmountHeader = ({
  sortMode,
  sortDirection,
  onSortModeChange,
  onSortDirectionChange,
  setCurrentPage,
}: {
  sortMode: CoinSortMode;
  sortDirection: boolean;
  onSortModeChange: (mode: CoinSortMode) => void;
  onSortDirectionChange: (ascending: boolean) => void;
  setCurrentPage: (page: number) => void;
}) => (
  <Button
    className='px-0'
    variant='link'
    onClick={() => {
      if (sortMode === 'amount') {
        onSortDirectionChange(!sortDirection);
      } else {
        onSortModeChange('amount');
        onSortDirectionChange(false);
      }
      setCurrentPage(0);
    }}
  >
    <span className='text-foreground hover:underline'>
      <Trans>Amount</Trans>
    </span>
    {sortMode === 'amount' ? (
      sortDirection ? (
        <ArrowUp className='ml-2 h-4 w-4' aria-hidden='true' />
      ) : (
        <ArrowDown className='ml-2 h-4 w-4' aria-hidden='true' />
      )
    ) : (
      <span className='ml-2 w-4 h-4' />
    )}
  </Button>
);

const AmountCell = ({
  row,
  precision,
}: {
  row: Row<CoinRecord>;
  precision: number;
}) => (
  <div className='font-mono truncate'>
    <NumberFormat
      value={fromMojos(row.getValue('amount') as string, precision)}
      minimumFractionDigits={0}
      maximumFractionDigits={precision}
    />
  </div>
);

const ConfirmedHeader = ({
  sortMode,
  sortDirection,
  onSortModeChange,
  onSortDirectionChange,
  setCurrentPage,
}: {
  sortMode: CoinSortMode;
  sortDirection: boolean;
  onSortModeChange: (mode: CoinSortMode) => void;
  onSortDirectionChange: (ascending: boolean) => void;
  setCurrentPage: (page: number) => void;
}) => (
  <Button
    className='px-0'
    variant='link'
    onClick={() => {
      if (sortMode === 'created_height') {
        onSortDirectionChange(!sortDirection);
      } else {
        onSortModeChange('created_height');
        onSortDirectionChange(false);
      }
      setCurrentPage(0);
    }}
  >
    <Trans>Confirmed</Trans>
    {sortMode === 'created_height' ? (
      sortDirection ? (
        <ArrowUp className='ml-2 h-4 w-4' aria-hidden='true' />
      ) : (
        <ArrowDown className='ml-2 h-4 w-4' aria-hidden='true' />
      )
    ) : (
      <span className='ml-2 w-4 h-4' />
    )}
  </Button>
);

const ConfirmedCell = ({ row }: { row: Row<CoinRecord> }) =>
  row.original.created_timestamp
    ? formatTimestamp(row.original.created_timestamp, 'short', 'short')
    : row.original.created_height
      ? row.original.created_height.toString()
      : t`Pending...`;

const ClawbackHeader = ({
  sortMode,
  sortDirection,
  onSortModeChange,
  onSortDirectionChange,
  setCurrentPage,
}: {
  sortMode: CoinSortMode;
  sortDirection: boolean;
  onSortModeChange: (mode: CoinSortMode) => void;
  onSortDirectionChange: (ascending: boolean) => void;
  setCurrentPage: (page: number) => void;
}) => (
  <Button
    className='px-0'
    variant='link'
    onClick={() => {
      if (sortMode === 'clawback_timestamp') {
        onSortDirectionChange(!sortDirection);
      } else {
        onSortModeChange('clawback_timestamp');
        onSortDirectionChange(false);
      }
      setCurrentPage(0);
    }}
  >
    <Trans>Clawback Expiration</Trans>
    {sortMode === 'clawback_timestamp' ? (
      sortDirection ? (
        <ArrowUp className='ml-2 h-4 w-4' aria-hidden='true' />
      ) : (
        <ArrowDown className='ml-2 h-4 w-4' aria-hidden='true' />
      )
    ) : (
      <span className='ml-2 w-4 h-4' />
    )}
  </Button>
);

const ClawbackCell = ({ row }: { row: Row<CoinRecord> }) =>
  row.original.clawback_timestamp
    ? formatTimestamp(row.original.clawback_timestamp, 'short', 'short')
    : t`No expiration`;

const SpentHeader = ({
  sortMode,
  sortDirection,
  includeSpentCoins,
  onSortModeChange,
  onSortDirectionChange,
  onIncludeSpentCoinsChange,
  setCurrentPage,
}: {
  sortMode: CoinSortMode;
  sortDirection: boolean;
  includeSpentCoins: boolean;
  onSortModeChange: (mode: CoinSortMode) => void;
  onSortDirectionChange: (ascending: boolean) => void;
  onIncludeSpentCoinsChange: (include: boolean) => void;
  setCurrentPage: (page: number) => void;
}) => (
  <div className='flex items-center space-x-1'>
    <Button
      className='px-0'
      variant='link'
      onClick={() => {
        if (sortMode === 'spent_height') {
          onSortDirectionChange(!sortDirection);
        } else {
          onSortModeChange('spent_height');
          onSortDirectionChange(false);
        }
        setCurrentPage(0);
      }}
    >
      <Trans>Spent</Trans>
      {sortMode === 'spent_height' ? (
        sortDirection ? (
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
        const newIncludeSpentCoins = !includeSpentCoins;
        onIncludeSpentCoinsChange(newIncludeSpentCoins);
        if (newIncludeSpentCoins) {
          onSortModeChange('spent_height');
          onSortDirectionChange(false);
        } else {
          onSortModeChange('created_height');
          onSortDirectionChange(false);
        }
        setCurrentPage(0);
      }}
      aria-label={includeSpentCoins ? t`Hide spent coins` : t`Show spent coins`}
    >
      {includeSpentCoins ? (
        <FilterXIcon className='h-3 w-3' aria-hidden='true' />
      ) : (
        <FilterIcon className='h-3 w-3' aria-hidden='true' />
      )}
    </Button>
  </div>
);

const SpentCell = ({ row }: { row: Row<CoinRecord> }) =>
  row.original.spent_timestamp
    ? formatTimestamp(row.original.spent_timestamp, 'short', 'short')
    : row.original.spent_height
      ? row.original.spent_height.toString()
      : row.original.transaction_id
        ? t`Pending...`
        : row.original.offer_id
          ? t`Locked in offer`
          : '';

// Wrapper components to eliminate remaining inline arrow functions
const CoinIdHeaderWrapper = (props: CoinListProps) => (
  <CoinIdHeader
    sortMode={props.sortMode}
    sortDirection={props.sortDirection}
    onSortModeChange={props.onSortModeChange}
    onSortDirectionChange={props.onSortDirectionChange}
    setCurrentPage={props.setCurrentPage}
  />
);

const AmountHeaderWrapper = (props: CoinListProps) => (
  <AmountHeader
    sortMode={props.sortMode}
    sortDirection={props.sortDirection}
    onSortModeChange={props.onSortModeChange}
    onSortDirectionChange={props.onSortDirectionChange}
    setCurrentPage={props.setCurrentPage}
  />
);

const ConfirmedHeaderWrapper = (props: CoinListProps) => (
  <ConfirmedHeader
    sortMode={props.sortMode}
    sortDirection={props.sortDirection}
    onSortModeChange={props.onSortModeChange}
    onSortDirectionChange={props.onSortDirectionChange}
    setCurrentPage={props.setCurrentPage}
  />
);

const SpentHeaderWrapper = (props: CoinListProps) => (
  <SpentHeader
    sortMode={props.sortMode}
    sortDirection={props.sortDirection}
    includeSpentCoins={props.includeSpentCoins}
    onSortModeChange={props.onSortModeChange}
    onSortDirectionChange={props.onSortDirectionChange}
    onIncludeSpentCoinsChange={props.onIncludeSpentCoinsChange}
    setCurrentPage={props.setCurrentPage}
  />
);

const ClawbackHeaderWrapper = (props: CoinListProps) => (
  <ClawbackHeader
    sortMode={props.sortMode}
    sortDirection={props.sortDirection}
    onSortModeChange={props.onSortModeChange}
    onSortDirectionChange={props.onSortDirectionChange}
    setCurrentPage={props.setCurrentPage}
  />
);

const AmountCellWrapper = ({
  row,
  precision,
}: {
  row: Row<CoinRecord>;
  precision: number;
}) => <AmountCell row={row} precision={precision} />;

// Factory function to create columns outside the component
const createColumns = (props: CoinListProps): ColumnDef<CoinRecord>[] => [
  {
    id: 'select',
    size: 30,
    header: SelectAllHeader,
    cell: SelectRowCell,
    enableSorting: false,
  },
  {
    accessorKey: 'coin_id',
    size: 100,
    header: () => <CoinIdHeaderWrapper {...props} />,
    cell: CoinIdCell,
  },
  {
    accessorKey: 'amount',
    size: 100,
    header: () => <AmountHeaderWrapper {...props} />,
    cell: ({ row }: { row: Row<CoinRecord> }) => (
      <AmountCellWrapper row={row} precision={props.precision} />
    ),
  },
  {
    accessorKey: 'created_height',
    header: () => <ConfirmedHeaderWrapper {...props} />,
    size: 140,
    cell: ConfirmedCell,
  },
  props.clawback
    ? {
        accessorKey: 'clawback_timestamp',
        header: () => <ClawbackHeaderWrapper {...props} />,
        size: 120,
        cell: ClawbackCell,
      }
    : {
        accessorKey: 'spent_height',
        header: () => <SpentHeaderWrapper {...props} />,
        size: 120,
        cell: SpentCell,
      },
];

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
  clawback: boolean;
}

export default function CoinList(props: CoinListProps) {
  // Use the factory function to create columns
  const columns = useMemo(() => createColumns(props), [props]);

  const getRowStyles = (row: Row<CoinRecord>) => {
    const coin = row.original;
    const isSpent = !!coin.spent_height || !!coin.transaction_id;
    const isSelected = row.getIsSelected();

    let className = '';

    if (isSelected) {
      className += ' bg-accent';
    }

    if (isSpent) {
      className += ' opacity-50 relative';
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
      <div className='flex-shrink-0 pt-4'>
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

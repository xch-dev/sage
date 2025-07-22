import { DataTable } from '@/components/ui/data-table';
import { t } from '@lingui/core/macro';
import { Row, SortingState } from '@tanstack/react-table';
import BigNumber from 'bignumber.js';
import { motion } from 'framer-motion';
import { useState } from 'react';
import { TransactionRecord } from '../bindings';
import { Loading } from './Loading';
import { columns, FlattenedTransaction } from './TransactionColumns';
import { getAssetDisplayName } from '@/lib/utils';

export function TransactionListView({
  transactions,
  onSortingChange,
  isLoading = false,
  summarized = true,
}: {
  transactions: TransactionRecord[];
  onSortingChange?: (ascending: boolean) => void;
  isLoading?: boolean;
  summarized?: boolean;
}) {
  const [sorting, setSorting] = useState<SortingState>([]);

  const flattenedTransactions = transactions.flatMap((transaction) => {
    const created = transaction.created.map(
      (coin): FlattenedTransaction => ({
        type: coin.asset.kind,
        address: coin.address,
        displayName: getAssetDisplayName(
          coin.asset.name,
          coin.asset.ticker,
          coin.asset.kind,
        ),
        itemId: coin.asset.asset_id ?? 'XCH',
        amount: coin.amount.toString(),
        transactionHeight: transaction.height,
        iconUrl: coin.asset.icon_url ?? '',
        timestamp: transaction.timestamp,
        precision: coin.asset.precision,
      }),
    );

    const spent = transaction.spent.map(
      (coin): FlattenedTransaction => ({
        type: coin.asset.kind,
        address: coin.address,
        displayName: getAssetDisplayName(
          coin.asset.name,
          coin.asset.ticker,
          coin.asset.kind,
        ),
        itemId: coin.asset.asset_id ?? 'XCH',
        amount: BigNumber(coin.amount).negated().toString(),
        transactionHeight: transaction.height,
        iconUrl: coin.asset.icon_url ?? '',
        timestamp: transaction.timestamp,
        precision: coin.asset.precision,
      }),
    );

    if (!summarized) {
      return [...created, ...spent];
    }

    // For summarized view, combine created and spent coins
    const allCoins = [...created, ...spent];

    // Group coins by item_id and calculate net amounts
    const summaryMap = new Map<string, FlattenedTransaction>();

    allCoins.forEach((coin) => {
      const existing = summaryMap.get(coin.itemId);
      const amount = BigNumber(coin.amount);

      if (existing) {
        summaryMap.set(coin.itemId, {
          ...existing,
          amount: BigNumber(existing.amount).plus(amount).toString(),
        });
      } else {
        summaryMap.set(coin.itemId, {
          ...coin,
          amount: amount.toString(),
        });
      }
    });

    // Convert the map to an array and format the amounts
    return [...summaryMap.values()];
  });

  // Function to determine if a row is the first in a transaction group
  const getRowStyles = (row: Row<FlattenedTransaction>) => {
    const currentHeight = row.original.transactionHeight;
    const rowIndex = flattenedTransactions.indexOf(row.original);

    // If it's not the first row, check if the previous row has a different transaction height
    if (rowIndex > 0) {
      const prevHeight = flattenedTransactions[rowIndex - 1].transactionHeight;

      // If this row has a different height than the previous one, it's the start of a new transaction group
      if (currentHeight !== prevHeight) {
        return {
          className:
            'border-t-2 border-t-neutral-300 dark:border-t-neutral-700',
        };
      }
    }

    return {};
  };

  return (
    <motion.div
      role='region'
      aria-label={t`Transaction history table`}
      initial={{ opacity: 0.6 }}
      animate={{ opacity: 1 }}
      transition={{ duration: 0.3 }}
    >
      {isLoading ? (
        <div className='flex justify-center items-center py-12'>
          <Loading size={40} text={t`Loading transactions...`} />
        </div>
      ) : (
        <DataTable
          columns={columns}
          data={flattenedTransactions}
          getRowStyles={getRowStyles}
          onSortingChange={(updatedSort) => {
            setSorting(updatedSort);
            if (typeof updatedSort === 'function') {
              const newSort = updatedSort([]);
              if (newSort.length > 0 && newSort[0].id === 'transactionHeight') {
                onSortingChange?.(newSort[0].desc === false);
              }
            } else if (
              updatedSort.length > 0 &&
              updatedSort[0].id === 'transactionHeight'
            ) {
              onSortingChange?.(updatedSort[0].desc === false);
            }
          }}
          state={{ sorting }}
        />
      )}
    </motion.div>
  );
}

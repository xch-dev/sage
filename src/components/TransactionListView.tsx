import { DataTable } from '@/components/ui/data-table';
import { nftUri } from '@/lib/nftUri';
import { t } from '@lingui/core/macro';
import { Row, SortingState } from '@tanstack/react-table';
import BigNumber from 'bignumber.js';
import { motion } from 'framer-motion';
import { useState } from 'react';
import { TransactionCoin, TransactionRecord } from '../bindings';
import { Loading } from './Loading';
import { columns, FlattenedTransaction } from './TransactionColumns';

function getDisplayName(coin: TransactionCoin) {
  switch (coin.type) {
    case 'xch':
      return 'Chia';
    case 'cat':
      return coin.name ?? 'Unknown CAT';
    case 'did':
      return coin.name ? `${coin.name}` : 'Unknown DID';
    case 'nft':
      return coin.name ? `${coin.name}` : 'Unknown NFT';
    default:
      return coin.type === 'unknown' ? 'Unknown' : coin.type.toUpperCase();
  }
}

function getItemId(coin: TransactionCoin) {
  switch (coin.type) {
    case 'xch':
      return 'xch';
    case 'cat':
      return coin.asset_id;
    case 'did':
      return coin.launcher_id;
    case 'nft':
      return coin.launcher_id;
    default:
      return coin.type;
  }
}

function getIconUrl(coin: TransactionCoin) {
  switch (coin.type) {
    case 'xch':
      return 'https://icons.dexie.space/xch.webp';
    case 'cat':
      return coin.icon_url;
    case 'nft':
      return nftUri(coin.icon ? 'image/png' : null, coin.icon);
    default:
      return null;
  }
}

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
    const created: FlattenedTransaction[] = transaction.created.map((coin) => ({
      type: coin.type,
      address: coin.address,
      displayName: getDisplayName(coin),
      itemId: getItemId(coin),
      amount: coin.amount.toString(),
      transactionHeight: transaction.height,
      iconUrl: getIconUrl(coin),
      timestamp: transaction.timestamp,
    }));

    const spent: FlattenedTransaction[] = transaction.spent.map((coin) => ({
      type: coin.type,
      address: coin.address,
      displayName: getDisplayName(coin),
      itemId: getItemId(coin),
      amount: BigNumber(coin.amount).negated().toString(),
      transactionHeight: transaction.height,
      iconUrl: getIconUrl(coin),
      timestamp: transaction.timestamp,
    }));

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

import { DataTable } from '@/components/ui/data-table';
import { nftUri } from '@/lib/nftUri';
import { t } from '@lingui/core/macro';
import { SortingState } from '@tanstack/react-table';
import { motion } from 'framer-motion';
import { useState } from 'react';
import { TransactionCoin, TransactionRecord } from '../bindings';
import { Loading } from './Loading';
import { columns } from './TransactionColumns';

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
    const created = transaction.created.map((coin) => ({
      ...coin,
      displayName: getDisplayName(coin),
      item_id: getItemId(coin),
      amount: `+${coin.amount.toString()}`,
      transactionHeight: transaction.height,
      icon_url: getIconUrl(coin),
      timestamp: transaction.timestamp,
    }));

    const spent = transaction.spent.map((coin) => ({
      ...coin,
      displayName: getDisplayName(coin),
      item_id: getItemId(coin),
      amount: `-${coin.amount.toString()}`,
      transactionHeight: transaction.height,
      icon_url: getIconUrl(coin),
      timestamp: transaction.timestamp,
    }));

    if (!summarized) {
      return [...created, ...spent];
    }

    if (transaction.height === 6787085) {
      // For summarized view, combine created and spent coins
      const allCoins = [...created, ...spent];

      // Group coins by item_id and calculate net amounts
      const summaryMap = new Map();

      allCoins.forEach((coin) => {
        const existing = summaryMap.get(coin.item_id);
        const amount = parseInt(coin.amount);

        if (existing) {
          summaryMap.set(coin.item_id, {
            ...existing,
            amount: existing.amount + amount,
          });
        } else {
          summaryMap.set(coin.item_id, {
            ...coin,
            amount,
          });
        }
      });

      // Convert the map to an array and format the amounts
      return Array.from(summaryMap.values()).map((coin) => ({
        ...coin,
        amount: coin.amount > 0 ? `+${coin.amount}` : coin.amount.toString(),
      }));
    }

    return [];
  });

  // Function to determine if a row is the first in a transaction group
  const getRowStyles = (row: any) => {
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

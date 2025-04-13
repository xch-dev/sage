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
      return coin.type.toUpperCase();
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
      amount: `+${coin.amount.toString()}`,
      transactionHeight: transaction.height,
      icon_url: getIconUrl(coin),
      timestamp: transaction.timestamp,
    }));

    const spent = transaction.spent.map((coin) => ({
      ...coin,
      displayName: getDisplayName(coin),
      amount: `-${coin.amount.toString()}`,
      transactionHeight: transaction.height,
      icon_url: getIconUrl(coin),
      timestamp: transaction.timestamp,
    }));

    if (!summarized) {
      return [...created, ...spent];
    }

    // For summarized view, group by coin type and net the amounts
    const coinGroups = new Map<string, Array<(typeof created)[0]>>();

    // Group created coins by type and ticker
    created.forEach((coin) => {
      // Use type as the key for grouping, add displayName for uniqueness
      const key = `${coin.type}_${coin.displayName}`;
      if (!coinGroups.has(key)) {
        coinGroups.set(key, []);
      }
      const group = coinGroups.get(key);
      if (group) {
        group.push(coin);
      }
    });

    // Group spent coins by type and ticker
    spent.forEach((coin) => {
      // Use type as the key for grouping, add displayName for uniqueness
      const key = `${coin.type}_${coin.displayName}`;
      if (!coinGroups.has(key)) {
        coinGroups.set(key, []);
      }
      const group = coinGroups.get(key);
      if (group) {
        group.push(coin);
      }
    });

    // Net amounts for each group
    const summarizedCoins: Array<(typeof created)[0]> = [];

    coinGroups.forEach((coins) => {
      // Skip if there's only one coin and it's not worth summarizing
      if (coins.length === 1) {
        summarizedCoins.push(coins[0]);
        return;
      }

      // Calculate net amount
      let netAmount = BigInt(0);
      let hasSent = false;
      let hasReceived = false;
      const isNftOrDid = coins.some(
        (coin) =>
          coin.type.toLowerCase() === 'nft' ||
          coin.type.toLowerCase() === 'did',
      );

      coins.forEach((coin) => {
        const amountStr = coin.amount.replace(/[+]/g, '').replace(/-/g, '');
        const amount = BigInt(amountStr);
        if (coin.amount.startsWith('+')) {
          netAmount += amount;
          hasReceived = true;
        } else {
          netAmount -= amount;
          hasSent = true;
        }
      });

      // Create a summarized coin
      const baseCoin = coins[0];

      // Special handling for NFT and DID transactions with both sent and received coins
      let netAmountStr;
      if (isNftOrDid && hasSent && hasReceived) {
        netAmountStr = 'edited';
      } else {
        netAmountStr =
          netAmount > 0
            ? `+${netAmount.toString()}`
            : netAmount < 0
              ? `-${(-netAmount).toString()}`
              : '0';
      }

      summarizedCoins.push({
        ...baseCoin,
        amount: netAmountStr,
        // Use the first coin's ID as a representative
        coin_id: `${baseCoin.coin_id}_summarized`,
      });
    });

    return summarizedCoins;
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

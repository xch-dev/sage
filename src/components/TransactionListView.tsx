import { TransactionCoin, TransactionRecord } from '../bindings';
import { DataTable } from '@/components/ui/data-table';
import { columns } from './TransactionColumns';
import { SortingState } from '@tanstack/react-table';
import { useState } from 'react';
import { t } from '@lingui/core/macro';
import { Loading } from './Loading';
import { motion } from 'framer-motion';
import { nftUri } from '@/lib/nftUri';

function getDisplayName(coin: TransactionCoin) {
  switch (coin.type) {
    case 'xch':
      return 'XCH';
    case 'cat':
      return coin.ticker ?? 'CAT';
    case 'did':
      return coin.name ? `DID: ${coin.name}` : 'DID';
    case 'nft':
      return coin.name ? `NFT: ${coin.name}` : 'NFT';
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
      return nftUri(coin.image_mime_type, coin.image_data);
    default:
      return null;
  }
}

export function TransactionListView({
  transactions,
  onSortingChange,
  isLoading = false,
}: {
  transactions: TransactionRecord[];
  onSortingChange?: (ascending: boolean) => void;
  isLoading?: boolean;
}) {
  const [sorting, setSorting] = useState<SortingState>([]);

  const flattenedTransactions = transactions.flatMap((transaction) => {
    const created = transaction.created.map((coin) => ({
      ...coin,
      displayName: getDisplayName(coin),
      amount: `+${coin.amount.toString()}`,
      transactionHeight: transaction.height,
      icon_url: getIconUrl(coin),
    }));

    const spent = transaction.spent.map((coin) => ({
      ...coin,
      displayName: getDisplayName(coin),
      amount: `-${coin.amount.toString()}`,
      transactionHeight: transaction.height,
      icon_url: getIconUrl(coin),
    }));

    return [...created, ...spent];
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

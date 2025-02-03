import { TransactionRecord } from '../bindings';
import { DataTable } from '@/components/ui/data-table';
import { columns } from './TransactionColumns';
import { cn } from '@/lib/utils';
import { SortingState } from '@tanstack/react-table';
import { useState } from 'react';
import { t } from '@lingui/core/macro';

export function TransactionListView({
  transactions,
  onSortingChange,
}: {
  transactions: TransactionRecord[];
  onSortingChange?: (ascending: boolean) => void;
}) {
  const [sorting, setSorting] = useState<SortingState>([]);

  const flattenedTransactions = transactions.flatMap((transaction) => {
    const created = transaction.created.map((coin) => ({
      ...coin,
      ticker: coin?.type === 'cat' ? coin?.name : null,
      amount: `+${coin.amount.toString()}`,
      transactionHeight: transaction.height,
    }));

    const spent = transaction.spent.map((coin) => ({
      ...coin,
      amount: `-${coin.amount.toString()}`,
      transactionHeight: transaction.height,
    }));

    return [...created, ...spent];
  });

  return (
    <div role="region" aria-label={t`Transaction history table`}>
      <DataTable
        columns={columns}
        data={flattenedTransactions}
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
        getRowStyles={(row) => ({
          className: cn(
            'transition-colors text-sm',
            'bg-white dark:bg-neutral-950',
            'focus:outline-none focus:ring-2 focus:ring-blue-500',
            flattenedTransactions.findIndex(
              (tx) =>
                tx.transactionHeight === row.original.transactionHeight &&
                tx.coin_id === row.original.coin_id,
            ) === 0 &&
              'border-t border-neutral-200 dark:border-neutral-700',
          ),
        })}
      />
    </div>
  );
}

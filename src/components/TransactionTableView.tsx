import { TransactionRecord } from '../bindings';
import { DataTable } from '@/components/ui/data-table';
import { columns } from './TransactionColumns';
import { cn } from '@/lib/utils';
import { SortingState } from '@tanstack/react-table';
import { useState } from 'react';

export function TransactionTableView({
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

  // Group transactions by height to determine alternating backgrounds
  const transactionGroups = new Set(flattenedTransactions.map(tx => tx.transactionHeight));
  const isEvenGroup = new Map([...transactionGroups].map((height, i) => [height, i % 2 === 0]));

  return (
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
        } else if (updatedSort.length > 0 && updatedSort[0].id === 'transactionHeight') {
          onSortingChange?.(updatedSort[0].desc === false);
        }
      }}
      state={{ sorting }}
      getRowStyles={(row) => ({
        className: cn(
          'transition-colors',
          // Make background colors more distinct between groups
          isEvenGroup.get(row.original.transactionHeight) 
            ? 'bg-neutral-100 dark:bg-neutral-800' 
            : 'bg-white dark:bg-neutral-950',
          // Add more prominent top border and padding to first row in each group
          flattenedTransactions.findIndex(tx => 
            tx.transactionHeight === row.original.transactionHeight && 
            tx.coin_id === row.original.coin_id
          ) === 0 && 'border-t-[6px] border-t-neutral-200 dark:border-t-neutral-700 pt-2'
        )
      })}
    />
  );
}

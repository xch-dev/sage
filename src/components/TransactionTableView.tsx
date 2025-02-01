import { TransactionRecord } from '../bindings';
import { DataTable } from '@/components/ui/data-table';
import { columns } from './TransactionColumns';

export function TransactionTableView({
  transactions,
}: {
  transactions: TransactionRecord[];
}) {
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

  return <DataTable columns={columns} data={flattenedTransactions} />;
}

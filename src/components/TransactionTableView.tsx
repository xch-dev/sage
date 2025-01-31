import { TransactionRecord } from '../bindings';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table';
import { t } from '@lingui/macro';
import { NumberFormat } from './NumberFormat';
import { fromMojos } from '@/lib/utils';
import { Link, useNavigate } from 'react-router-dom';
import { HeartPulseIcon } from 'lucide-react';

interface TransactionTableViewProps {
  transactions: TransactionRecord[];
}

export function TransactionTableView({ transactions }: TransactionTableViewProps) {
  const navigate = useNavigate();

  const flattenedTransactions = transactions.flatMap(transaction => {
    const created = transaction.created.map(coin => ({
      ...coin,
      transactionType: 'received' as const,
      transactionHeight: transaction.height,
    }));
    
    const spent = transaction.spent.map(coin => ({
      ...coin,
      transactionType: 'sent' as const,
      transactionHeight: transaction.height,
    }));

    return [...created, ...spent];
  });

  return (
    <Table>
      <TableHeader>
        <TableRow>
          <TableHead>{t`Block #`}</TableHead>
          <TableHead>{t`Ticker`}</TableHead>
          <TableHead>{t`Type`}</TableHead>
          <TableHead className='text-right w-[140px]'>{t`Amount`}</TableHead>
          <TableHead>{t`Address`}</TableHead>
          <TableHead>{t`Address Kind`}</TableHead>
          <TableHead>{t`Coin ID`}</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {flattenedTransactions.map((coin) => (
          <TableRow 
            key={coin.coin_id}
            className="cursor-pointer hover:bg-neutral-50 dark:hover:bg-neutral-900"
            onClick={() => navigate(`/transactions/${coin.transactionHeight}`)}
          >
            <TableCell>
              <Link 
                to={`/transactions/${coin.transactionHeight}`}
                className='hover:underline'
                onClick={(e) => e.stopPropagation()}
              >
                #{coin.transactionHeight}
              </Link>
            </TableCell>
            <TableCell>
              {coin.type === 'xch' ? 'XCH' : coin.type === 'cat' ? coin.ticker ?? 'CAT' : coin.type.toUpperCase()}
            </TableCell>
            <TableCell>
              {coin.transactionType === 'received' ? t`Received` : t`Sent`}
            </TableCell>
            <TableCell className='text-right'>
              <span className={coin.transactionType === 'received' ? 'text-green-600' : 'text-red-600'}>
                {coin.transactionType === 'received' ? '+' : '-'}
                <NumberFormat
                  value={fromMojos(coin.amount, 12)}
                  minimumFractionDigits={0}
                  maximumFractionDigits={12}
                />
              </span>
            </TableCell>
            <TableCell className='font-mono text-sm'>
              {coin.address?.slice(0, 10)}...
            </TableCell>
            <TableCell>
              {coin.address_kind}
            </TableCell>
            <TableCell className='font-mono text-sm'>
              {coin.coin_id.slice(0, 10)}...
            </TableCell>
          </TableRow>
        ))}
      </TableBody>
    </Table>
  );
} 
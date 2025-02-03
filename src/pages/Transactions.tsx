import Container from '@/components/Container';
import Header from '@/components/Header';
import { ReceiveAddress } from '@/components/ReceiveAddress';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { useErrors } from '@/hooks/useErrors';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { Info } from 'lucide-react';
import { useCallback, useEffect, useState } from 'react';
import {
  commands,
  events,
  PendingTransactionRecord,
  TransactionRecord,
} from '../bindings';
import { TransactionListView } from '@/components/TransactionListView';
import { TransactionCardView } from '@/components/TransactionCardView';
import { TransactionOptions } from '@/components/TransactionOptions';
import { useTransactionsParams } from '@/hooks/useTransactionsParams';

export function Transactions() {
  const { addError } = useErrors();
  const [params, setParams] = useTransactionsParams();
  const { page, pageSize, viewMode, search, ascending } = params;
  const [_pending, setPending] = useState<PendingTransactionRecord[]>([]);
  const [transactions, setTransactions] = useState<TransactionRecord[]>([]);
  const [totalTransactions, setTotalTransactions] = useState(0);
  const [isLoading] = useState(false);

  const updateTransactions = useCallback(async () => {
    commands
      .getPendingTransactions({})
      .then((data) => setPending(data.transactions))
      .catch(addError);

    commands
      .getTransactions({
        offset: (page - 1) * pageSize,
        limit: pageSize,
        ascending,
        find_value: search || null,
      })
      .then((data) => {
        setTransactions(data.transactions);
        setTotalTransactions(data.total);
      })
      .catch(addError);
  }, [addError, page, pageSize, ascending, search]);

  useEffect(() => {
    updateTransactions();

    const unlisten = events.syncEvent.listen((data) => {
      switch (data.payload.type) {
        case 'coin_state':
        case 'cat_info':
        case 'did_info':
        case 'nft_data':
        case 'puzzle_batch_synced':
          updateTransactions();
      }
    });

    return () => {
      unlisten.then((u) => u());
    };
  }, [updateTransactions]);

  return (
    <>
      <Header title={t`Transactions`}>
        <ReceiveAddress />
      </Header>
      <Container>
        {transactions.length === 0 && (
          <Alert className='mb-4'>
            <Info className='h-4 w-4' />
            <AlertTitle>
              <Trans>Note</Trans>
            </AlertTitle>
            <AlertDescription>
              <Trans>You have not made any transactions yet.</Trans>
            </AlertDescription>
          </Alert>
        )}

        <TransactionOptions
          params={params}
          onParamsChange={setParams}
          total={totalTransactions}
          isLoading={isLoading}
          className='mb-4'
        />

        {viewMode === 'list' ? (
          <TransactionCardView transactions={transactions} />
        ) : (
          <TransactionListView
            transactions={transactions}
            onSortingChange={(value) => setParams({ ascending: value })}
          />
        )}
      </Container>
    </>
  );
}

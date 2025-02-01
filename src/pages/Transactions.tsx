import Container from '@/components/Container';
import Header from '@/components/Header';
import { ReceiveAddress } from '@/components/ReceiveAddress';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { useErrors } from '@/hooks/useErrors';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { Info } from 'lucide-react';
import { useCallback, useEffect, useState } from 'react';
import { useSearchParams } from 'react-router-dom';
import {
  commands,
  events,
  PendingTransactionRecord,
  TransactionRecord,
} from '../bindings';

import { Pagination } from '@/components/Pagination';
import { useLocalStorage } from 'usehooks-ts';
import { ViewToggle, ViewMode } from '@/components/ViewToggle';
import { TransactionTableView } from '../components/TransactionTableView';
import { TransactionCardView } from '../components/TransactionCardView';

export function Transactions() {
  const { addError } = useErrors();

  const [params, setParams] = useSearchParams();
  const page = parseInt(params.get('page') ?? '1');
  const setPage = (page: number) => setParams({ page: page.toString() });

  const [pageSize, setPageSize] = useLocalStorage('transactionsPageSize', 8);
  const [view, setView] = useLocalStorage<ViewMode>('transactionsView', 'list');
  const [ascending, setAscending] = useState(true);

  // TODO: Show pending transactions
  const [_pending, setPending] = useState<PendingTransactionRecord[]>([]);
  const [transactions, setTransactions] = useState<TransactionRecord[]>([]);
  const [total, setTotal] = useState(0);

  const updateTransactions = useCallback(async () => {
    commands
      .getPendingTransactions({})
      .then((data) => setPending(data.transactions))
      .catch(addError);
    
    commands
      .getTransactionsEx({
        offset: (page - 1) * pageSize,
        limit: pageSize,
        ascending,
        find_value: 'HOA',
      })
      .then((data) => {
        setTransactions(data.transactions);
        setTotal(data.total);
      })
      .catch(addError);
  }, [addError, page, pageSize, ascending]);

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

  const handleSortingChange = (newAscending: boolean) => {
    setAscending(newAscending);
  };

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

        <div className='flex items-center justify-between mb-4'>
          <Pagination
            total={total}
            page={page}
            onPageChange={setPage}
            pageSize={pageSize}
            onPageSizeChange={setPageSize}
          />
          <ViewToggle view={view} onChange={setView} />
        </div>

        {view === 'list' ? (
          <TransactionCardView transactions={transactions} />
        ) : (
          <TransactionTableView
            transactions={transactions}
            onSortingChange={handleSortingChange}
          />
        )}

      </Container>
    </>
  );
}

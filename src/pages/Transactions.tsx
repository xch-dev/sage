import Container from '@/components/Container';
import Header from '@/components/Header';
import { ReceiveAddress } from '@/components/ReceiveAddress';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { useErrors } from '@/hooks/useErrors';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { Info, } from 'lucide-react';
import { useCallback, useEffect, useState } from 'react';
import { useSearchParams } from 'react-router-dom';
import {
  commands,
  events,
  PendingTransactionRecord,
  TransactionRecord,
} from '../bindings';
import { useLocalStorage } from 'usehooks-ts';
import { ViewMode } from '@/components/ViewToggle';
import { TransactionTableView } from '../components/TransactionTableView';
import { TransactionCardView } from '../components/TransactionCardView';
import { TransactionOptions } from '@/components/TransactionOptions';

export function Transactions() {
  const { addError } = useErrors();

  const [params, setParams] = useSearchParams();
  const page = parseInt(params.get('page') ?? '1');
  const search = params.get('search') ?? '';
  const setPage = (page: number) =>
    setParams({ ...Object.fromEntries(params), page: page.toString() });

  const [pageSize, setPageSize] = useLocalStorage('transactionsPageSize', 8);
  const [view, setView] = useLocalStorage<ViewMode>('transactionsView', 'list');
  const [ascending, setAscending] = useState(false);

  // TODO: Show pending transactions
  const [_pending, setPending] = useState<PendingTransactionRecord[]>([]);
  const [transactions, setTransactions] = useState<TransactionRecord[]>([]);
  const [total, setTotal] = useState(0);

  const [query, setQuery] = useState('');
  const [totalTransactions, setTotalTransactions] = useState(0);
  const [isLoading, setIsLoading] = useState(false);
  const [hasMoreTransactions, setHasMoreTransactions] = useState(true);

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
        setTotal(data.total);
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

  const handleSortingChange = (newAscending: boolean) => {
    setAscending(newAscending);
  };

  const handleSearch = (value: string) => {
    setParams({ ...Object.fromEntries(params), search: value, page: '1' });
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

        <TransactionOptions
          query={query}
          setQuery={setQuery}
          page={page}
          setPage={setPage}
          pageSize={pageSize}
          setPageSize={setPageSize}
          total={totalTransactions}
          isLoading={isLoading}
          view={view}
          setView={setView}
          ascending={ascending}
          setAscending={setAscending}
          handleSearch={handleSearch}
          className="mb-4"
        />

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

import Container from '@/components/Container';
import Header from '@/components/Header';
import { ReceiveAddress } from '@/components/ReceiveAddress';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Input } from '@/components/ui/input';
import { useErrors } from '@/hooks/useErrors';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { Info, SearchIcon, XIcon } from 'lucide-react';
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
import { Button } from '@/components/ui/button';

export function Transactions() {
  const { addError } = useErrors();

  const [params, setParams] = useSearchParams();
  const page = parseInt(params.get('page') ?? '1');
  const search = params.get('search') ?? '';
  const setPage = (page: number) => setParams({ ...Object.fromEntries(params), page: page.toString() });

  const [pageSize, setPageSize] = useLocalStorage('transactionsPageSize', 8);
  const [view, setView] = useLocalStorage<ViewMode>('transactionsView', 'list');
  const [ascending, setAscending] = useState(false);

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
        find_value: search || null,
      })
      .then((data) => {
        setTransactions(data.transactions);
        setTotal(data.total);
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

        <div className='space-y-4 mb-4'>
          <div className='flex items-center gap-2'>
            <div className='relative flex-1'>
              <SearchIcon
                className='absolute left-2 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground'
                aria-hidden='true'
              />
              <Input
                type='search'
                value={search}
                aria-label={t`Search transactions`}
                placeholder={t`Search transactions...`}
                onChange={(e) => handleSearch(e.target.value)}
                className='w-full pl-8 pr-8'
              />
              {search && (
                <Button
                  variant='ghost'
                  size='icon'
                  title={t`Clear search`}
                  aria-label={t`Clear search`}
                  className='absolute right-0 top-0 h-full px-2 hover:bg-transparent'
                  onClick={() => handleSearch('')}
                >
                  <XIcon className='h-4 w-4' aria-hidden='true' />
                </Button>
              )}
            </div>
            <ViewToggle view={view} onChange={setView} />
          </div>

          <div className='flex'>
            <Pagination
              total={total}
              page={page}
              onPageChange={setPage}
              pageSize={pageSize}
              onPageSizeChange={setPageSize}
            />
          </div>
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

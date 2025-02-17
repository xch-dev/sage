import Container from '@/components/Container';
import Header from '@/components/Header';
import { ReceiveAddress } from '@/components/ReceiveAddress';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { useErrors } from '@/hooks/useErrors';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { Info } from 'lucide-react';
import { useCallback, useEffect, useState, useRef } from 'react';
import {
  commands,
  events,
  PendingTransactionRecord,
  TransactionRecord,
} from '../bindings';
import { TransactionListView } from '@/components/TransactionListView';
import { TransactionOptions } from '@/components/TransactionOptions';
import { useTransactionsParams } from '@/hooks/useTransactionsParams';
import { useIntersectionObserver } from '@/hooks/useIntersectionObserver';
import { Pagination } from '@/components/Pagination';

export function Transactions() {
  const { addError } = useErrors();
  const [params, setParams] = useTransactionsParams();
  const { page, pageSize, search, ascending } = params;
  const [_pending, setPending] = useState<PendingTransactionRecord[]>([]);
  const [transactions, setTransactions] = useState<TransactionRecord[]>([]);
  const [totalTransactions, setTotalTransactions] = useState(0);
  const [isLoading] = useState(false);
  const optionsRef = useRef<HTMLDivElement>(null);
  const [isOptionsVisible, setIsOptionsVisible] = useState(true);
  const listRef = useRef<HTMLDivElement>(null);

  useIntersectionObserver(optionsRef, ([entry]) => {
    setIsOptionsVisible(entry.isIntersecting);
  });

  const updateTransactions = useCallback(async () => {
    commands
      .getPendingTransactions({})
      .then((data) => setPending(data.transactions))
      .catch(addError);

    // if the search term might be a block height, try to get the block
    // and add it to the list of transactions
    const searchHeight = search ? parseInt(search, 10) : null;
    const isValidHeight =
      searchHeight !== null && !isNaN(searchHeight) && searchHeight >= 0;

    let specificBlock: TransactionRecord[] = [];
    if (isValidHeight) {
      const block = await commands.getTransaction({ height: searchHeight });
      specificBlock = [block.transaction];
    }

    commands
      .getTransactions({
        offset: (page - 1) * pageSize,
        limit: pageSize,
        ascending,
        find_value: search || null,
      })
      .then((data) => {
        const combinedTransactions = [...specificBlock, ...data.transactions];
        setTransactions(combinedTransactions);
        setTotalTransactions(data.total + specificBlock.length);
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

  const renderPagination = useCallback(
    (compact: boolean = false) => (
      <Pagination
        page={page}
        total={totalTransactions}
        pageSize={pageSize}
        onPageChange={(newPage) => {
          setParams({ page: newPage });
          listRef.current?.scrollIntoView({ behavior: 'smooth' });
        }}
        onPageSizeChange={(newSize) =>
          setParams({ pageSize: newSize, page: 1 })
        }
        pageSizeOptions={[10, 25, 50, 100]}
        compact={compact}
        isLoading={isLoading}
      />
    ),
    [page, pageSize, totalTransactions, setParams, isLoading],
  );

  return (
    <>
      <Header
        title={t`Transactions`}
        paginationControls={
          !isOptionsVisible ? renderPagination(true) : undefined
        }
      >
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

        <div ref={optionsRef}>
          <TransactionOptions
            params={params}
            onParamsChange={setParams}
            total={totalTransactions}
            isLoading={isLoading}
            className='mb-4'
            renderPagination={() => renderPagination(false)}
          />
        </div>

        <div ref={listRef}>
          <TransactionListView
            transactions={transactions}
            onSortingChange={(value) =>
              setParams({ ascending: value, page: 1 })
            }
          />
        </div>
      </Container>
    </>
  );
}

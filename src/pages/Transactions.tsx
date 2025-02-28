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
import { Loading } from '@/components/Loading';
import { motion, AnimatePresence } from 'framer-motion';

export function Transactions() {
  const { addError } = useErrors();
  const [params, setParams] = useTransactionsParams();
  const { page, pageSize, search, ascending } = params;
  const [_pending, setPending] = useState<PendingTransactionRecord[]>([]);
  const [transactions, setTransactions] = useState<TransactionRecord[]>([]);
  const [totalTransactions, setTotalTransactions] = useState(0);
  const [isLoading, setIsLoading] = useState(false);
  const optionsRef = useRef<HTMLDivElement>(null);
  const [isOptionsVisible, setIsOptionsVisible] = useState(true);
  const listRef = useRef<HTMLDivElement>(null);
  const [isPaginationLoading, setIsPaginationLoading] = useState(false);

  useIntersectionObserver(optionsRef, ([entry]) => {
    setIsOptionsVisible(entry.isIntersecting);
  });

  const updateTransactions = useCallback(async () => {
    setIsLoading(true);

    try {
      const pendingResult = await commands.getPendingTransactions({});
      setPending(pendingResult.transactions);

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

      const result = await commands.getTransactions({
        offset: (page - 1) * pageSize,
        limit: pageSize,
        ascending,
        find_value: search || null,
      });

      const combinedTransactions = [...specificBlock, ...result.transactions];
      setTransactions(combinedTransactions);
      setTotalTransactions(result.total + specificBlock.length);
    } catch (error) {
      addError(error as any);
    } finally {
      setIsLoading(false);
      setIsPaginationLoading(false);
    }
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

  const handlePageChange = useCallback((newPage: number, compact?: boolean) => {
    setIsPaginationLoading(true);
    setParams({ page: newPage });
    if (compact) {
      listRef.current?.scrollIntoView({
        behavior: 'smooth',
        block: 'start',
      });
    }
  }, [setParams]);

  const handlePageSizeChange = useCallback((newSize: number, compact?: boolean) => {
    setIsPaginationLoading(true);
    setParams({ pageSize: newSize, page: 1 });
    if (compact) {
        listRef.current?.scrollIntoView({
        behavior: 'smooth',
        block: 'start',
      });
    }
  }, [setParams]);

  const renderPagination = useCallback(
    (compact: boolean = false) => (
      <Pagination
        page={page}
        total={totalTransactions}
        pageSize={pageSize}
        onPageChange={(newPage) => handlePageChange(newPage, compact)}
        onPageSizeChange={(newSize) => handlePageSizeChange(newSize, compact)}
        pageSizeOptions={[10, 25, 50]}
        compact={compact}
        isLoading={isLoading || isPaginationLoading}
      />
    ),
    [page, pageSize, totalTransactions, isLoading, isPaginationLoading, handlePageChange, handlePageSizeChange],
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
        {transactions.length === 0 && !isLoading && !isPaginationLoading && (
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
            onParamsChange={(newParams) => {
              if (
                newParams.page !== params.page ||
                newParams.pageSize !== params.pageSize
              ) {
                setIsPaginationLoading(true);
              }
              setParams(newParams);
            }}
            total={totalTransactions}
            isLoading={isLoading || isPaginationLoading}
            className='mb-4'
            renderPagination={() => renderPagination(false)}
          />
        </div>

        <div ref={listRef} className='relative'>
          <AnimatePresence>
            {isPaginationLoading && (
              <motion.div
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                exit={{ opacity: 0 }}
                className='absolute inset-0 flex items-center justify-center bg-background/80 backdrop-blur-sm z-10 rounded-md'
                style={{ minHeight: '200px' }}
              >
                <Loading size={40} text={t`Loading transactions...`} />
              </motion.div>
            )}
          </AnimatePresence>

          <TransactionListView
            transactions={transactions}
            onSortingChange={(value) => {
              setIsPaginationLoading(true);
              setParams({ ascending: value, page: 1 });
            }}
            isLoading={isLoading && !isPaginationLoading}
          />
        </div>
      </Container>
    </>
  );
}

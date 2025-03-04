import { useSearchParams } from 'react-router-dom';
import { useLocalStorage } from 'usehooks-ts';
import { ViewMode } from '@/components/ViewToggle';

const TRANSACTION_PAGE_SIZE_STORAGE_KEY = 'sage-wallet-transaction-page-size';
const TRANSACTION_SORT_MODE_STORAGE_KEY = 'sage-wallet-transaction-sort-mode';

export interface TransactionParams {
  page: number;
  pageSize: number;
  search: string;
  ascending: boolean;
}

export type SetTransactionParams = (params: Partial<TransactionParams>) => void;

export function useTransactionsParams(): [
  TransactionParams,
  SetTransactionParams,
] {
  const [params, setParams] = useSearchParams();

  const [storedSortMode, setStoredSortMode] = useLocalStorage<boolean>(
    TRANSACTION_SORT_MODE_STORAGE_KEY,
    false,
  );

  const [storedPageSize, setStoredPageSize] = useLocalStorage<number>(
    TRANSACTION_PAGE_SIZE_STORAGE_KEY,
    10,
  );

  const page = parseInt(params.get('page') ?? '1');
  const search = params.get('search') ?? '';
  const ascending = params.get('ascending')
    ? params.get('ascending') === 'true'
    : storedSortMode;
  const pageSize = parseInt(
    params.get('pageSize') ?? storedPageSize.toString(),
  );

  const updateParams = ({
    page,
    pageSize,
    search,
    ascending,
  }: Partial<TransactionParams>) => {
    setParams(
      (prev) => {
        const next = new URLSearchParams(prev);

        if (page !== undefined) {
          next.set('page', page.toString());
        }

        if (pageSize !== undefined) {
          next.set('pageSize', pageSize.toString());
          setStoredPageSize(pageSize);
        }

        if (search !== undefined) {
          if (search) {
            next.set('search', search);
          } else {
            next.delete('search');
          }
        }

        if (ascending !== undefined) {
          next.set('ascending', ascending.toString());
          setStoredSortMode(ascending);
        }

        return next;
      },
      { replace: true },
    );
  };

  return [
    {
      page,
      pageSize,
      search,
      ascending,
    },
    updateParams,
  ];
}

import { OfferFindSide, OfferRecordStatus, OfferSortMode } from '@/bindings';
import { useCallback, useMemo } from 'react';
import { useSearchParams } from 'react-router-dom';
import { useLocalStorage } from 'usehooks-ts';

const OFFER_STATUS_STORAGE_KEY = 'sage-offer-filter';
const OFFER_SIDE_STORAGE_KEY = 'sage-offer-find-side';
const OFFER_SORT_STORAGE_KEY = 'sage-offer-sort';
const OFFER_ASCENDING_STORAGE_KEY = 'sage-offer-ascending';
const OFFER_PAGE_SIZE_STORAGE_KEY = 'sage-offer-page-size';

export type OfferStatusFilter = OfferRecordStatus | 'all';

export interface OfferParams {
  page: number;
  pageSize: number;
  query: string | null;
  status: OfferStatusFilter;
  findSide: OfferFindSide;
  sort: OfferSortMode;
  ascending: boolean;
}

export type SetOfferParams = (params: Partial<OfferParams>) => void;

export function useOfferParams(): [OfferParams, SetOfferParams] {
  const [searchParams, setSearchParams] = useSearchParams();
  const [status, setStatus] = useLocalStorage<OfferStatusFilter>(
    OFFER_STATUS_STORAGE_KEY,
    'all',
  );
  const [findSide, setFindSide] = useLocalStorage<OfferFindSide>(
    OFFER_SIDE_STORAGE_KEY,
    'any',
  );
  const [sort, setSort] = useLocalStorage<OfferSortMode>(
    OFFER_SORT_STORAGE_KEY,
    'created',
  );
  const [ascending, setAscending] = useLocalStorage<boolean>(
    OFFER_ASCENDING_STORAGE_KEY,
    false,
  );
  const [pageSize, setPageSize] = useLocalStorage<number>(
    OFFER_PAGE_SIZE_STORAGE_KEY,
    24,
  );

  const params = useMemo(() => {
    const pageParam = searchParams.get('page');
    const pageNumber = pageParam ? Number(pageParam) : 1;
    const page =
      Number.isInteger(pageNumber) && pageNumber > 0 ? pageNumber : 1;

    const queryParam = searchParams.get('query');
    const query = queryParam && queryParam.trim() !== '' ? queryParam : null;

    return { page, pageSize, query, status, findSide, sort, ascending };
  }, [searchParams, pageSize, status, findSide, sort, ascending]);

  const setParams = useCallback(
    (newParams: Partial<OfferParams>) => {
      const updated = { ...params, ...newParams };

      if (newParams.status !== undefined) setStatus(newParams.status);
      if (newParams.findSide !== undefined) setFindSide(newParams.findSide);
      if (newParams.sort !== undefined) setSort(newParams.sort);
      if (newParams.ascending !== undefined) setAscending(newParams.ascending);
      if (newParams.pageSize !== undefined) setPageSize(newParams.pageSize);

      setSearchParams(
        {
          ...(updated.page > 1 && { page: updated.page.toString() }),
          ...(updated.query && { query: updated.query }),
        },
        { replace: true },
      );
    },
    [
      params,
      setSearchParams,
      setStatus,
      setFindSide,
      setSort,
      setAscending,
      setPageSize,
    ],
  );

  return [params, setParams];
}

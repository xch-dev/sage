import { useCallback, useMemo } from 'react';
import { useSearchParams } from 'react-router-dom';
import { useLocalStorage } from 'usehooks-ts';

const NFT_HIDDEN_STORAGE_KEY = 'sage-wallet-nft-hidden';
const NFT_GROUP_STORAGE_KEY = 'sage-wallet-nft-group';
const NFT_SORT_STORAGE_KEY = 'sage-wallet-nft-sort';
const NFT_PAGE_SIZE_STORAGE_KEY = 'sage-wallet-nft-page-size';
const NFT_CARD_SIZE_STORAGE_KEY = 'sage-wallet-nft-card-size';

export enum NftSortMode {
  Name = 'name',
  Recent = 'recent',
}

export enum NftGroupMode {
  None = 'none',
  Collection = 'collection',
  OwnerDid = 'owner_did',
  MinterDid = 'minter_did',
}

export enum CardSize {
  Large = 'large',
  Small = 'small',
}

export interface NftParams {
  pageSize: number;
  page: number;
  sort: NftSortMode;
  group: NftGroupMode;
  showHidden: boolean;
  query: string | null;
  cardSize: CardSize;
}

export type SetNftParams = (params: Partial<NftParams>) => void;

export function useNftParams(): [NftParams, SetNftParams] {
  const [searchParams, setSearchParams] = useSearchParams();
  const [sort, setSort] = useLocalStorage<NftSortMode>(
    NFT_SORT_STORAGE_KEY,
    NftSortMode.Name,
  );
  const [showHidden, setShowHidden] = useLocalStorage<boolean>(
    NFT_HIDDEN_STORAGE_KEY,
    false,
  );
  const [group, setGroup] = useLocalStorage<NftGroupMode>(
    NFT_GROUP_STORAGE_KEY,
    NftGroupMode.None,
  );
  const [pageSize, setPageSize] = useLocalStorage<number>(
    NFT_PAGE_SIZE_STORAGE_KEY,
    24,
  );
  const [cardSize, setCardSize] = useLocalStorage<CardSize>(
    NFT_CARD_SIZE_STORAGE_KEY,
    CardSize.Large,
  );

  const params = useMemo(
    () => {
      // Validate page parameter - ensure it's a positive integer
      const pageParam = searchParams.get('page');
      const pageNumber = pageParam ? Number(pageParam) : 1;
      const validPage = Number.isInteger(pageNumber) && pageNumber > 0 ? pageNumber : 1;

      // Validate query parameter - ensure it's not an empty string
      const queryParam = searchParams.get('query');
      const validQuery = queryParam && queryParam.trim() !== '' ? queryParam : null;

      return {
        pageSize,
        page: validPage,
        sort,
        group,
        showHidden,
        query: validQuery,
        cardSize,
      };
    },
    [searchParams, sort, group, showHidden, pageSize, cardSize],
  );

  const setParams = useCallback(
    (newParams: Partial<NftParams>) => {
      const updatedParams = { ...params, ...newParams };

      if (newParams.sort !== undefined) {
        setSort(newParams.sort);
      }

      if (newParams.showHidden !== undefined) {
        setShowHidden(newParams.showHidden);
      }

      if (newParams.group !== undefined) {
        setGroup(newParams.group);
      }

      if (newParams.pageSize !== undefined) {
        setPageSize(newParams.pageSize);
      }

      if (newParams.cardSize !== undefined) {
        setCardSize(newParams.cardSize);
      }

      setSearchParams(
        {
          ...(updatedParams.page > 1 && {
            page: updatedParams.page.toString(),
          }),
          ...(updatedParams.query && { query: updatedParams.query }),
        },
        { replace: true },
      );
    },
    [
      params,
      setSearchParams,
      setSort,
      setShowHidden,
      setGroup,
      setPageSize,
      setCardSize,
    ],
  );

  return [params, setParams];
}

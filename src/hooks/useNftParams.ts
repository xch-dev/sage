import { useSearchParams } from 'react-router-dom';
import { useLocalStorage } from 'usehooks-ts';
import { useCallback, useMemo } from 'react';

const NFT_HIDDEN_STORAGE_KEY = 'sage-wallet-nft-hidden';
const NFT_GROUP_STORAGE_KEY = 'sage-wallet-nft-group';
const NFT_SORT_STORAGE_KEY = 'sage-wallet-nft-sort';
const NFT_PAGE_SIZE_STORAGE_KEY = 'sage-wallet-nft-page-size';
const NFT_CARD_SIZE_STORAGE_KEY = 'sage-wallet-nft-card-size';

export enum NftView {
  Name = 'name',
  Recent = 'recent',
  Collection = 'collection',
  Did = 'did',
}

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
  const [view, setView] = useLocalStorage<NftSortMode>(
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
    () => ({
      pageSize,
      page: Number(searchParams.get('page') || 1),
      sort: view,
      group,
      showHidden,
      query: searchParams.get('query'),
      cardSize,
    }),
    [searchParams, view, group, showHidden, pageSize, cardSize],
  );

  const setParams = useCallback(
    (newParams: Partial<NftParams>) => {
      const updatedParams = { ...params, ...newParams };

      if (newParams.sort !== undefined) {
        setView(newParams.sort);
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
    [params, setSearchParams, setView, setShowHidden, setGroup, setPageSize, setCardSize],
  );

  return [params, setParams];
}

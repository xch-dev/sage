import { useSearchParams } from 'react-router-dom';
import { useLocalStorage } from 'usehooks-ts';

const NFT_VIEW_STORAGE_KEY = 'sage-wallet-nft-view';
const NFT_HIDDEN_STORAGE_KEY = 'sage-wallet-nft-hidden';

export enum NftView {
  Name = 'name',
  Recent = 'recent',
  Collection = 'collection',
}

export interface NftParams {
  pageSize: number;
  page: number;
  view: NftView;
  showHidden: boolean;
  query?: string;
}

export type SetNftParams = (params: Partial<NftParams>) => void;

function parseView(view: string | null): NftView {
  switch (view) {
    case 'name':
      return NftView.Name;
    case 'recent':
      return NftView.Recent;
    case 'collection':
      return NftView.Collection;
    default:
      return NftView.Name;
  }
}

export function useNftParams(): [NftParams, SetNftParams] {
  const [params, setParams] = useSearchParams();
  const [storedView, setStoredView] = useLocalStorage<NftView>(
    NFT_VIEW_STORAGE_KEY,
    NftView.Name
  );
  const [storedShowHidden, setStoredShowHidden] = useLocalStorage<boolean>(
    NFT_HIDDEN_STORAGE_KEY,
    false
  );

  const pageSize = parseInt(params.get('pageSize') ?? '12');
  const page = parseInt(params.get('page') ?? '1');
  const view = parseView(params.get('view') ?? storedView);
  const showHidden = (params.get('showHidden') ?? storedShowHidden.toString()) === 'true';
  const query = params.get('query') ?? undefined;

  const updateParams = ({
    pageSize,
    page,
    view,
    showHidden,
    query,
  }: Partial<NftParams>) => {
    setParams(
      (prev) => {
        const next = new URLSearchParams(prev);

        if (pageSize !== undefined) {
          next.set('pageSize', pageSize.toString());
        }

        if (page !== undefined) {
          next.set('page', page.toString());
        }

        if (view !== undefined) {
          next.set('view', view);
          setStoredView(view);
        }

        if (showHidden !== undefined) {
          next.set('showHidden', showHidden.toString());
          setStoredShowHidden(showHidden);
        }

        if (query !== undefined) {
          if (query) {
            next.set('query', query);
          } else {
            next.delete('query');
          }
        }

        return next;
      },
      { replace: true },
    );
  };

  return [{ pageSize, page, view, showHidden, query }, updateParams];
}

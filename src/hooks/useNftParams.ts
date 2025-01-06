import { useSearchParams } from 'react-router-dom';

export interface NftParams {
  pageSize: number;
  page: number;
  view: NftView;
  showHidden: boolean;
  query: string;
}

export enum NftView {
  Name = 'name',
  Recent = 'recent',
  Collection = 'collection',
}

export function parseView(view: string): NftView {
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

export type SetNftParams = (params: Partial<NftParams>) => void;

export function useNftParams(): [NftParams, SetNftParams] {
  const [params, setParams] = useSearchParams();

  const pageSize = parseInt(params.get('pageSize') ?? '12');
  const page = parseInt(params.get('page') ?? '1');
  const view = parseView(params.get('view') ?? 'recent');
  const showHidden = (params.get('showHidden') ?? 'false') === 'true';
  const query = params.get('query') ?? '';

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
        }

        if (showHidden !== undefined) {
          next.set('showHidden', showHidden.toString());
        }

        if (query !== undefined) {
          next.set('query', query);
        }

        return next;
      },
      { replace: true },
    );
  };

  return [{ pageSize, page, view, showHidden, query }, updateParams];
}

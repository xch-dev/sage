import { useSearchParams } from 'react-router-dom';
import { useLocalStorage } from 'usehooks-ts';
import { ViewMode } from '@/components/ViewToggle';

const ZERO_BALANCE_STORAGE_KEY = 'sage-wallet-show-zero-balance';
const TOKEN_SORT_STORAGE_KEY = 'sage-wallet-token-sort';
const TOKEN_VIEW_MODE_STORAGE_KEY = 'sage-wallet-token-view-mode';

export interface TokenParams {
  viewMode: ViewMode;
  sortMode: TokenSortMode;
  showHidden: boolean;
  search: string;
  showZeroBalance: boolean;
}

export enum TokenSortMode {
  Name = 'name',
  Balance = 'balance',
}

export function parseSortMode(view: string): TokenSortMode {
  switch (view) {
    case 'name':
      return TokenSortMode.Name;
    case 'balance':
      return TokenSortMode.Balance;
    default:
      return TokenSortMode.Name;
  }
}

export type SetTokenParams = (params: Partial<TokenParams>) => void;

export function useTokenParams(): [TokenParams, SetTokenParams] {
  const [params, setParams] = useSearchParams();

  const [storedShowZeroBalance, setStoredShowZeroBalance] =
    useLocalStorage<boolean>(ZERO_BALANCE_STORAGE_KEY, false);

  const [storedTokenView, setStoredTokenView] = useLocalStorage<TokenSortMode>(
    TOKEN_SORT_STORAGE_KEY,
    TokenSortMode.Name,
  );

  const [storedViewMode, setStoredViewMode] = useLocalStorage<ViewMode>(
    TOKEN_VIEW_MODE_STORAGE_KEY,
    'grid'
  );

  const sortMode = parseSortMode(params.get('sortMode') ?? storedTokenView);
  const showHidden = (params.get('showHidden') ?? 'false') === 'true';
  const showZeroBalance =
    (params.get('showZeroBalance') ?? storedShowZeroBalance.toString()) ===

    'true';
  const search = params.get('search') ?? '';

  const viewMode = params.get('viewMode') as ViewMode ?? storedViewMode;

  const updateParams = ({
    sortMode,
    showHidden,
    showZeroBalance,
    search,
    viewMode,
  }: Partial<TokenParams>) => {
    setParams(
      (prev) => {
        const next = new URLSearchParams(prev);

        if (sortMode !== undefined) {
          next.set('sortMode', sortMode);
          setStoredTokenView(sortMode);
        }


        if (showHidden !== undefined) {
          next.set('showHidden', showHidden.toString());
        }

        if (showZeroBalance !== undefined) {
          next.set('showZeroBalance', showZeroBalance.toString());
          setStoredShowZeroBalance(showZeroBalance);
        }

        if (search !== undefined) {
          if (search) {
            next.set('search', search);
          } else {
            next.delete('search');
          }
        }

        if (viewMode !== undefined) {
          next.set('viewMode', viewMode);
          setStoredViewMode(viewMode);
        }

        return next;
      },
      { replace: true },
    );
  };

  return [{
    viewMode,
    sortMode,
    showHidden,
    showZeroBalance,
    search
  }, updateParams];
}

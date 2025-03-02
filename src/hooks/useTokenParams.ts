import { useSearchParams } from 'react-router-dom';
import { useLocalStorage } from 'usehooks-ts';
import { ViewMode } from '@/components/ViewToggle';

const ZERO_BALANCE_STORAGE_KEY = 'sage-wallet-show-zero-balance';
const TOKEN_SORT_STORAGE_KEY = 'sage-wallet-token-sort';
const TOKEN_VIEW_MODE_STORAGE_KEY = 'sage-wallet-token-view-mode';
const HIDDEN_CATS_STORAGE_KEY = 'sage-wallet-show-hidden-cats';

export interface TokenParams {
  viewMode: ViewMode;
  sortMode: TokenSortMode;
  showZeroBalanceTokens: boolean;
  showHiddenCats: boolean;
  search: string;
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
    'grid',
  );

  const [storedShowHiddenCats, setStoredShowHiddenCats] =
    useLocalStorage<boolean>(HIDDEN_CATS_STORAGE_KEY, false);

  const sortMode = parseSortMode(params.get('sortMode') ?? storedTokenView);
  const showZeroBalanceTokens =
    (params.get('showZeroBalanceTokens') ??
      storedShowZeroBalance.toString()) === 'true';
  const showHiddenCats =
    (params.get('showHiddenCats') ?? storedShowHiddenCats.toString()) ===
    'true';
  const search = params.get('search') ?? '';

  const viewMode = (params.get('viewMode') as ViewMode) ?? storedViewMode;

  const updateParams = ({
    sortMode,
    showZeroBalanceTokens,
    showHiddenCats,
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

        if (showZeroBalanceTokens !== undefined) {
          next.set('showZeroBalanceTokens', showZeroBalanceTokens.toString());
          setStoredShowZeroBalance(showZeroBalanceTokens);
        }

        if (showHiddenCats !== undefined) {
          next.set('showHiddenCats', showHiddenCats.toString());
          setStoredShowHiddenCats(showHiddenCats);
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

  return [
    {
      viewMode,
      sortMode,
      showZeroBalanceTokens,
      showHiddenCats,
      search,
    },
    updateParams,
  ];
}

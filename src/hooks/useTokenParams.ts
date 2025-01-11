import { useSearchParams } from 'react-router-dom';
import { useLocalStorage } from 'usehooks-ts';

const ZERO_BALANCE_STORAGE_KEY = 'sage-wallet-show-zero-balance';
const TOKEN_SORT_STORAGE_KEY = 'sage-wallet-token-sort';

export interface TokenParams {
  view: TokenView;
  showHidden: boolean;
  search: string;
  showZeroBalance: boolean;
}

export enum TokenView {
  Name = 'name',
  Balance = 'balance',
}

export function parseView(view: string): TokenView {
  switch (view) {
    case 'name':
      return TokenView.Name;
    case 'balance':
      return TokenView.Balance;
    default:
      return TokenView.Name;
  }
}

export type SetTokenParams = (params: Partial<TokenParams>) => void;

export function useTokenParams(): [TokenParams, SetTokenParams] {
  const [params, setParams] = useSearchParams();

  const [storedShowZeroBalance, setStoredShowZeroBalance] =
    useLocalStorage<boolean>(ZERO_BALANCE_STORAGE_KEY, false);

  const [storedTokenView, setStoredTokenView] = useLocalStorage<TokenView>(
    TOKEN_SORT_STORAGE_KEY,
    TokenView.Name,
  );

  const view = parseView(params.get('view') ?? storedTokenView);
  const showHidden = (params.get('showHidden') ?? 'false') === 'true';
  const showZeroBalance =
    (params.get('showZeroBalance') ?? storedShowZeroBalance.toString()) ===
    'true';
  const search = params.get('search') ?? '';

  const updateParams = ({
    view,
    showHidden,
    showZeroBalance,
    search,
  }: Partial<TokenParams>) => {
    setParams(
      (prev) => {
        const next = new URLSearchParams(prev);

        if (view !== undefined) {
          next.set('view', view);
          setStoredTokenView(view);
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

        return next;
      },
      { replace: true },
    );
  };

  return [{ view, showHidden, showZeroBalance, search }, updateParams];
}

import { useSearchParams } from 'react-router-dom';

export interface TokenParams {
  view: TokenView;
  showHidden: boolean;
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

  const view = parseView(params.get('view') ?? 'name');
  const showHidden = (params.get('showHidden') ?? 'false') === 'true';
  const showZeroBalance = (params.get('showZeroBalance') ?? 'true') === 'true';

  const updateParams = ({ view, showHidden, showZeroBalance }: Partial<TokenParams>) => {
    setParams(
      (prev) => {
        const next = new URLSearchParams(prev);

        if (view !== undefined) {
          next.set('view', view);
        }

        if (showHidden !== undefined) {
          next.set('showHidden', showHidden.toString());
        }

        if (showZeroBalance !== undefined) {
          next.set('showZeroBalance', showZeroBalance.toString());
        }

        return next;
      },
      { replace: true },
    );
  };

  return [{ view, showHidden, showZeroBalance }, updateParams];
}

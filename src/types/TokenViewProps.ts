import { CatRecord } from '../bindings';

export interface TokenViewProps {
  cats: Array<
    CatRecord & { balanceInUsd: number; sortValue: number; priceInUsd: number }
  >;
  xchBalance: string;
  xchDecimals: number;
  xchBalanceUsd: number;
  xchPrice: number;
}

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

export interface TokenRecord {
  asset_id: string;
  name: string | null;
  ticker: string | null;
  icon_url: string | null;
  balance: number | string;
  balanceInUsd: number;
  priceInUsd: number;
  decimals: number;
  isXch?: boolean;
  visible?: boolean;
}

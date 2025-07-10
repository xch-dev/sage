import { TokenRecord } from '../bindings';

export interface TokenViewProps {
  cats: Array<
    TokenRecord & {
      balanceInUsd: number;
      sortValue: number;
      priceInUsd: number;
    }
  >;
  xchRecord: UITokenRecord;
}

export interface UITokenRecord {
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

import { TokenRecord } from '../bindings';

export interface TokenViewProps {
  tokens: (PricedTokenRecord & { sortValue: number })[];
}

export interface PricedTokenRecord extends TokenRecord {
  balance: number | string;
  balanceInUsd: number;
  priceInUsd: number;
}

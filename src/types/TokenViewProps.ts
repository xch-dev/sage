import { TokenRecord } from '../bindings';

export interface TokenViewProps {
  cats: TokenRecordWithPrices[];
  xchRecord: TokenRecordWithPrices;
}

export interface TokenRecordWithPrices extends TokenRecord {
  balanceInUsd: number;
  priceInUsd: number;
  isXch: boolean;
  sortValue?: number;
}

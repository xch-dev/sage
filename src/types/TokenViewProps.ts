import { TokenRecord } from '../bindings';

export interface TokenViewProps {
  cats: TokenRecordWithPrices[];
  xchRecord: TokenRecordWithPrices;
}

// Keep the old interface for backward compatibility if needed
export interface TokenRecordWithPrices extends TokenRecord {
  // Override balance to be more flexible (number | string instead of Amount)
  balance: number | string;
  // Add price-related fields
  balanceInUsd: number;
  priceInUsd: number;
  decimals: number;
  isXch?: boolean;
  sortValue?: number;
}

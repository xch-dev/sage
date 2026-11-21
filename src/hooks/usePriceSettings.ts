import { useLocalStorage } from 'usehooks-ts';

/** Currencies that CoinGecko quotes XCH in. */
export const FIAT_CURRENCIES = [
  'USD',
  'EUR',
  'GBP',
  'JPY',
  'CNY',
  'CAD',
  'AUD',
  'CHF',
  'KRW',
  'INR',
  'BRL',
  'MXN',
] as const;

export type FiatCurrency = (typeof FIAT_CURRENCIES)[number];

export function usePriceSettings() {
  const [storedCurrency, setCurrency] = useLocalStorage<FiatCurrency>(
    'sage-fiat-currency',
    'USD',
  );
  const [pricesEnabled, setPricesEnabled] = useLocalStorage<boolean>(
    'sage-price-lookups',
    true,
  );

  // Fall back to USD if the stored value isn't a supported currency.
  const currency = FIAT_CURRENCIES.includes(storedCurrency)
    ? storedCurrency
    : 'USD';

  return { currency, setCurrency, pricesEnabled, setPricesEnabled };
}

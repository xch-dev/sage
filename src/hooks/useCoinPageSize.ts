import { useLocalStorage } from 'usehooks-ts';

const COIN_PAGE_SIZE_STORAGE_KEY = 'sage-wallet-coin-page-size';

export const COIN_PAGE_SIZE_OPTIONS = [10, 25, 50, 100];

export function useCoinPageSize() {
  const [storedPageSize, setPageSize] = useLocalStorage<number>(
    COIN_PAGE_SIZE_STORAGE_KEY,
    COIN_PAGE_SIZE_OPTIONS[0],
  );

  // Fall back to the default if the stored value is not a supported option
  const pageSize = COIN_PAGE_SIZE_OPTIONS.includes(storedPageSize)
    ? storedPageSize
    : COIN_PAGE_SIZE_OPTIONS[0];

  return { pageSize, setPageSize };
}

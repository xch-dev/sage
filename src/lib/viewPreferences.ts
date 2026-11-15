import { SIDEBAR_COLLAPSED_STORAGE_KEY } from '@/components/Layout';
import { COIN_PAGE_SIZE_STORAGE_KEY } from '@/hooks/useCoinPageSize';
import {
  NFT_CARD_SIZE_STORAGE_KEY,
  NFT_GROUP_STORAGE_KEY,
  NFT_HIDDEN_STORAGE_KEY,
  NFT_PAGE_SIZE_STORAGE_KEY,
  NFT_SORT_STORAGE_KEY,
} from '@/hooks/useNftParams';
import {
  OFFER_ASCENDING_STORAGE_KEY,
  OFFER_CARD_SIZE_STORAGE_KEY,
  OFFER_PAGE_SIZE_STORAGE_KEY,
  OFFER_SIDE_STORAGE_KEY,
  OFFER_SORT_STORAGE_KEY,
  OFFER_STATUS_STORAGE_KEY,
} from '@/hooks/useOfferParams';
import {
  HIDDEN_OPTIONS_STORAGE_KEY,
  OPTION_ASCENDING_STORAGE_KEY,
  OPTION_FILTER_STORAGE_KEY,
  OPTION_SORT_STORAGE_KEY,
  OPTION_VIEW_MODE_STORAGE_KEY,
} from '@/hooks/useOptionParams';
import {
  HIDDEN_CATS_STORAGE_KEY,
  TOKEN_SORT_STORAGE_KEY,
  TOKEN_VIEW_MODE_STORAGE_KEY,
  ZERO_BALANCE_STORAGE_KEY,
} from '@/hooks/useTokenParams';
import {
  TRANSACTION_PAGE_SIZE_STORAGE_KEY,
  TRANSACTION_SORT_MODE_STORAGE_KEY,
  TRANSACTION_VIEW_MODE_STORAGE_KEY,
} from '@/hooks/useTransactionsParams';

/** How lists are shown: sorting, filters, page and card sizes, view modes, and the sidebar. */
const VIEW_PREFERENCE_KEYS = [
  SIDEBAR_COLLAPSED_STORAGE_KEY,
  COIN_PAGE_SIZE_STORAGE_KEY,
  NFT_CARD_SIZE_STORAGE_KEY,
  NFT_GROUP_STORAGE_KEY,
  NFT_HIDDEN_STORAGE_KEY,
  NFT_PAGE_SIZE_STORAGE_KEY,
  NFT_SORT_STORAGE_KEY,
  OFFER_ASCENDING_STORAGE_KEY,
  OFFER_CARD_SIZE_STORAGE_KEY,
  OFFER_PAGE_SIZE_STORAGE_KEY,
  OFFER_SIDE_STORAGE_KEY,
  OFFER_SORT_STORAGE_KEY,
  OFFER_STATUS_STORAGE_KEY,
  HIDDEN_OPTIONS_STORAGE_KEY,
  OPTION_ASCENDING_STORAGE_KEY,
  OPTION_FILTER_STORAGE_KEY,
  OPTION_SORT_STORAGE_KEY,
  OPTION_VIEW_MODE_STORAGE_KEY,
  HIDDEN_CATS_STORAGE_KEY,
  TOKEN_SORT_STORAGE_KEY,
  TOKEN_VIEW_MODE_STORAGE_KEY,
  ZERO_BALANCE_STORAGE_KEY,
  TRANSACTION_PAGE_SIZE_STORAGE_KEY,
  TRANSACTION_SORT_MODE_STORAGE_KEY,
  TRANSACTION_VIEW_MODE_STORAGE_KEY,
];

/** Puts every list back the way it's shown by default. */
export function resetViewPreferences() {
  for (const key of VIEW_PREFERENCE_KEYS) {
    try {
      window.localStorage.removeItem(key);
    } catch {
      // Storage can be unavailable, in which case there's nothing to reset.
    }

    // Mounted `useLocalStorage` hooks listen for this, and go back to their defaults.
    window.dispatchEvent(new StorageEvent('local-storage', { key }));
  }
}

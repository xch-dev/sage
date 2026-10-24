import { usePrices } from '@/hooks/usePrices';
import { labelAsset } from '@/lib/tokenIndex';
import { useCallback } from 'react';

/**
 * Shows a CAT the way the token list and token page do: with an enabled token
 * index's suggested name, ticker and icon wherever the wallet has none. The same
 * rules as the token list apply everywhere a token is shown (transactions, coins,
 * offers, selectors): a value from the user or Dexie is never replaced, nothing
 * is written to the wallet, and NFTs, DIDs and options are left as they are.
 */
export function useAssetLabel() {
  const { getSuggestedLabel } = usePrices();

  return useCallback(
    <
      T extends {
        asset_id: string | null;
        name: string | null;
        ticker: string | null;
        icon_url: string | null;
        kind?: string;
      },
    >(
      asset: T,
    ): T => labelAsset(asset, getSuggestedLabel),
    [getSuggestedLabel],
  );
}

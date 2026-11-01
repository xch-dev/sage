import { PriceContext } from '@/contexts/PriceContext';
import { labelAsset } from '@/lib/tokenIndex';
import { useCallback, useContext } from 'react';

/**
 * Shows a CAT the way the token list and token page do: with an enabled token
 * index's suggested name, ticker and icon wherever the wallet has none. The same
 * rules as the token list apply everywhere a token is shown (transactions, coins,
 * offers, selectors): a value from the user or Dexie is never replaced, nothing
 * is written to the wallet, and NFTs, DIDs and options are left as they are.
 *
 * Outside a PriceProvider (the WalletConnect and app request dialogs render above
 * it) there are no suggestions, so the wallet's own label is shown unchanged.
 * Requiring the provider there threw on every signing prompt with a token in its
 * summary and blanked the whole window.
 */
export function useAssetLabel() {
  const getSuggestedLabel = useContext(PriceContext)?.getSuggestedLabel;

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
    ): T => (getSuggestedLabel ? labelAsset(asset, getSuggestedLabel) : asset),
    [getSuggestedLabel],
  );
}

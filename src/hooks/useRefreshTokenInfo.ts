import { commands } from '@/bindings';
import { usePrices } from '@/hooks/usePrices';
import { withRefreshedLabel } from '@/lib/priceSources';
import { useCallback } from 'react';

/**
 * Refresh Info for a CAT, from the first enabled source that can describe it
 * (Settings → Price & Token Sources). Dexie is read by the wallet as before; a
 * token index's label is saved to the wallet record, since the user asked for
 * it. Returns false when no enabled source has anything for the CAT.
 */
export function useRefreshTokenInfo() {
  const { getRefreshPlan } = usePrices();

  return useCallback(
    async (assetId: string) => {
      const plan = getRefreshPlan(assetId);

      switch (plan.kind) {
        case 'dexie':
          await commands.resyncCat({ asset_id: assetId });
          return true;
        case 'index': {
          const { token } = await commands.getToken({ asset_id: assetId });
          if (!token) return false;
          await commands.updateCat({
            record: withRefreshedLabel(token, plan.label),
          });
          return true;
        }
        case 'none':
          return false;
      }
    },
    [getRefreshPlan],
  );
}

import { GetFeeRateResponse } from '@/bindings';
import { FEE_RATES_MAX_AGE_MS, getFeeRates } from '@/lib/fees';
import { useEffect, useState } from 'react';

/**
 * The recommended and minimum fee rates, and the tiers' estimated rates, in mojos per cost,
 * once they're loaded. They're refreshed while the component is mounted, since the estimates
 * follow the mempool. They're only shown as hints, so a failure to load them leaves `null` (or
 * the last rates that loaded), and the request that needs a rate reports the error instead.
 */
export function useFeeRates(): GetFeeRateResponse | null {
  const [feeRates, setFeeRates] = useState<GetFeeRateResponse | null>(null);

  useEffect(() => {
    let cancelled = false;

    const load = () => {
      getFeeRates()
        .then((rates) => {
          if (!cancelled) setFeeRates(rates);
        })
        .catch(() => {
          // Left as it was; see above.
        });
    };

    load();
    // Checked twice as often as they go stale, so that a check landing just before they do
    // doesn't leave them for another whole period.
    const interval = setInterval(load, FEE_RATES_MAX_AGE_MS / 2);

    return () => {
      cancelled = true;
      clearInterval(interval);
    };
  }, []);

  return feeRates;
}

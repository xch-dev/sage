import { GetFeeRateResponse } from '@/bindings';
import { getFeeRates } from '@/lib/fees';
import { useEffect, useState } from 'react';

/**
 * The recommended and minimum fee rates, in mojos per cost, once they're loaded. They're only
 * shown as hints, so a failure to load them leaves `null`, and the request that needs a rate
 * reports the error instead.
 */
export function useFeeRates(): GetFeeRateResponse | null {
  const [feeRates, setFeeRates] = useState<GetFeeRateResponse | null>(null);

  useEffect(() => {
    let cancelled = false;

    getFeeRates()
      .then((rates) => {
        if (!cancelled) setFeeRates(rates);
      })
      .catch(() => {
        // Left as `null`; see above.
      });

    return () => {
      cancelled = true;
    };
  }, []);

  return feeRates;
}

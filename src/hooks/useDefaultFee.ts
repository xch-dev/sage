import { AUTO_FEE, isAutoFee } from '@/lib/fees';
import * as React from 'react';
import { useLocalStorage } from 'usehooks-ts';

export interface DefaultFee {
  fee: string;
}

const isValidFee = (value: string): boolean => {
  if (isAutoFee(value)) return true;
  const num = parseFloat(value);
  return !isNaN(num) && num >= 0;
};

const DEFAULT_FEE = AUTO_FEE;

/**
 * Where the default fee was stored before fees could be calculated automatically. Its
 * default was 0, which is now too low to be confirmed reliably, so only a positive fee
 * someone chose is carried over.
 */
const LEGACY_KEY = 'defaultFee';

function initialDefaultFee(): DefaultFee {
  try {
    const legacy = window.localStorage.getItem(LEGACY_KEY);
    const fee = legacy ? (JSON.parse(legacy) as DefaultFee).fee : undefined;

    if (fee && isValidFee(fee) && parseFloat(fee) > 0) {
      return { fee };
    }
  } catch {
    // An unreadable legacy value is ignored.
  }

  return { fee: DEFAULT_FEE };
}

/** The fee inputs start from: {@link AUTO_FEE}, or an amount in the network's unit. */
export function useDefaultFee() {
  const [defaultFee, setDefaultFee] = useLocalStorage<DefaultFee>(
    'defaultNetworkFee',
    initialDefaultFee,
  );

  const setFee = (fee: string) => {
    if (isValidFee(fee)) {
      setDefaultFee({ fee });
    }
  };

  // Ensure we always have a valid fee value
  React.useEffect(() => {
    if (!isValidFee(defaultFee.fee)) {
      setDefaultFee({ fee: DEFAULT_FEE });
    }
  }, [defaultFee.fee, setDefaultFee]);

  return {
    fee: isValidFee(defaultFee.fee) ? defaultFee.fee : DEFAULT_FEE,
    setFee,
  };
}

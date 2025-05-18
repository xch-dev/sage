import { useLocalStorage } from 'usehooks-ts';

export interface DefaultFee {
  fee: string;
}

const isValidFee = (value: string): boolean => {
  const num = parseFloat(value);
  return !isNaN(num) && num >= 0;
};

const DEFAULT_FEE = '0';

export function useDefaultFee() {
  const [defaultFee, setDefaultFee] = useLocalStorage<DefaultFee>(
    'defaultFee',
    { fee: DEFAULT_FEE },
  );

  const setFee = (fee: string) => {
    if (isValidFee(fee)) {
      setDefaultFee({ fee });
    }
  };

  // Ensure we always have a valid fee value
  if (!isValidFee(defaultFee.fee)) {
    setDefaultFee({ fee: DEFAULT_FEE });
  }

  return {
    fee: isValidFee(defaultFee.fee) ? defaultFee.fee : DEFAULT_FEE,
    setFee,
  };
}

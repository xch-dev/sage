import { useLocalStorage } from 'usehooks-ts';

export interface DefaultOfferExpiry {
  enabled: boolean;
  days: string;
  hours: string;
  minutes: string;
}

const validateNumber = (value: string): string => {
  if (!value) return '';
  const num = parseInt(value);
  if (isNaN(num) || num < 0) return '';
  return num.toString();
};

const validateExpiry = (expiry: DefaultOfferExpiry): DefaultOfferExpiry => {
  return {
    enabled: expiry.enabled,
    days: validateNumber(expiry.days),
    hours: validateNumber(expiry.hours),
    minutes: validateNumber(expiry.minutes),
  };
};

export function useDefaultOfferExpiry() {
  const [expiry, setExpiry] = useLocalStorage<DefaultOfferExpiry>(
    'default-offer-expiry',
    {
      enabled: false,
      days: '1',
      hours: '',
      minutes: '',
    },
  );

  // Ensure stored values are valid on load
  const validatedExpiry = validateExpiry(expiry);

  const setValidatedExpiry = (newExpiry: DefaultOfferExpiry) => {
    setExpiry(validateExpiry(newExpiry));
  };

  return {
    expiry: validatedExpiry,
    setExpiry: setValidatedExpiry,
  };
}

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

  // Calculate total seconds
  const getTotalSeconds = (): number | null => {
    if (!validatedExpiry.enabled) return null;

    // the || operates on the result of parseInt, so if parseInt returns NaN,
    // it will return 1 etc. because NaN is falsy
    const days = parseInt(validatedExpiry.days) || 1;
    const hours = parseInt(validatedExpiry.hours) || 0;
    const minutes = parseInt(validatedExpiry.minutes) || 0;

    return days * 24 * 60 * 60 + hours * 60 * 60 + minutes * 60;
  };

  return {
    expiry: validatedExpiry,
    setExpiry: setValidatedExpiry,
    getTotalSeconds,
  };
}

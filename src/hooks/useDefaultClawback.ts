import { useLocalStorage } from 'usehooks-ts';

export interface DefaultClawback {
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

const validateClawback = (clawback: DefaultClawback): DefaultClawback => {
  try {
    return {
      enabled: clawback.enabled,
      days: validateNumber(clawback.days),
      hours: validateNumber(clawback.hours),
      minutes: validateNumber(clawback.minutes),
    };
  } catch {
    return {
      enabled: false,
      days: '',
      hours: '1',
      minutes: '',
    };
  }
};

export function useDefaultClawback() {
  const [clawback, setClawback] = useLocalStorage<DefaultClawback>(
    'default-clawback',
    {
      enabled: false,
      days: '',
      hours: '1',
      minutes: '',
    },
  );

  // Ensure stored values are valid on load
  const validatedClawback = validateClawback(clawback);

  const setValidatedClawback = (newClawback: DefaultClawback) => {
    setClawback(validateClawback(newClawback));
  };

  return {
    clawback: validatedClawback,
    setClawback: setValidatedClawback,
  };
}

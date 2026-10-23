// A relative duration, optionally pinned to an absolute time picked by the user.
export interface Expiration {
  days: string;
  hours: string;
  minutes: string;
  // Absolute expiration (unix seconds) chosen from the date picker. When set,
  // it takes precedence over the relative days/hours/minutes duration.
  timestamp?: number;
}

export function nowSeconds(): number {
  return Math.ceil(Date.now() / 1000);
}

export function expirationDurationSeconds(expiration: Expiration): number {
  const days = parseInt(expiration.days) || 0;
  const hours = parseInt(expiration.hours) || 0;
  const minutes = parseInt(expiration.minutes) || 0;
  return days * 24 * 60 * 60 + hours * 60 * 60 + minutes * 60;
}

// The absolute expiration in unix seconds. A picked date is used exactly;
// otherwise the duration is counted from now.
export function expirationTimestamp(
  expiration: Expiration,
  now = nowSeconds(),
): number {
  return expiration.timestamp ?? now + expirationDurationSeconds(expiration);
}

// Splits the time remaining until a timestamp into days/hours/minutes,
// rounding up to the next minute so a future time never shows as zero.
export function durationUntil(
  timestamp: number,
  now = nowSeconds(),
): Pick<Expiration, 'days' | 'hours' | 'minutes'> {
  const totalMinutes = Math.max(0, Math.ceil((timestamp - now) / 60));
  return {
    days: Math.floor(totalMinutes / (24 * 60)).toString(),
    hours: Math.floor((totalMinutes % (24 * 60)) / 60).toString(),
    minutes: (totalMinutes % 60).toString(),
  };
}

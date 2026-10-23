import { t } from '@lingui/core/macro';

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

// The expiration to send when creating an offer, option or clawback. Unlike
// expirationTimestamp, which also drives display, this throws if the
// expiration is malformed or not in the future, as a backstop for callers.
export function futureExpirationTimestamp(
  expiration: Expiration,
  now = nowSeconds(),
): number {
  if (
    expiration.timestamp === undefined &&
    [expiration.days, expiration.hours, expiration.minutes].some(
      (value) => (parseInt(value) || 0) < 0,
    )
  ) {
    throw new Error(t`Expiration duration cannot be negative`);
  }

  const timestamp = expirationTimestamp(expiration, now);
  if (!Number.isSafeInteger(timestamp) || timestamp <= now) {
    throw new Error(t`Expiration must be in the future`);
  }

  return timestamp;
}

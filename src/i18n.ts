import { i18n } from '@lingui/core';
import BigNumber from 'bignumber.js';

export interface NumberFormatProps {
  value: string | number | BigNumber;
  style?: 'decimal' | 'currency' | 'percent';
  currency?: string;
  minimumFractionDigits?: number;
  maximumFractionDigits?: number;
}

/**
 * Short, locale-aware number for tight layouts (1.5K, 1.23M). Values below 1
 * keep three significant digits rather than rounding to zero.
 */
export function formatCompactNumber(
  value: string | number | BigNumber,
): string {
  const numberValue = new BigNumber(value).toNumber();

  if (numberValue !== 0 && Math.abs(numberValue) < 1) {
    return numberValue.toLocaleString(navigator.language, {
      maximumSignificantDigits: 3,
    });
  }

  return numberValue.toLocaleString(navigator.language, {
    notation: 'compact',
    maximumFractionDigits: 2,
  });
}

const RELATIVE_TIME_UNITS: [Intl.RelativeTimeFormatUnit, number][] = [
  ['year', 365 * 86400],
  ['month', 30 * 86400],
  ['day', 86400],
  ['hour', 3600],
  ['minute', 60],
];

/**
 * Narrow relative time for a unix timestamp in seconds ("in 3d", "2h ago"),
 * truncated to the largest whole unit.
 */
export function formatRelativeTime(
  timestamp: number,
  now: number = Date.now(),
): string {
  const deltaSeconds = timestamp - Math.floor(now / 1000);
  const formatter = new Intl.RelativeTimeFormat(navigator.language, {
    style: 'narrow',
  });

  for (const [unit, unitSeconds] of RELATIVE_TIME_UNITS) {
    if (Math.abs(deltaSeconds) >= unitSeconds) {
      return formatter.format(Math.trunc(deltaSeconds / unitSeconds), unit);
    }
  }

  return formatter.format(deltaSeconds, 'second');
}

/**
 * Load messages for requested locale and activate it.
 * This function isn't part of the LinguiJS library because there are
 * many ways how to load messages — from REST API, from file, from cache, etc.
 */
export async function loadCatalog(locale: string) {
  const { messages } = await import(`./locales/${locale}/messages.po`);
  i18n.loadAndActivate({ locale, messages });
}

export function formatNumber({
  value,
  style = 'decimal',
  currency,
  minimumFractionDigits,
  maximumFractionDigits,
}: NumberFormatProps): string {
  if (value == null) return '';

  try {
    const bigNumberValue = new BigNumber(value);
    if (bigNumberValue.isNaN()) return '';
    if (bigNumberValue.isGreaterThan(Number.MAX_SAFE_INTEGER))
      return value.toString();

    const numberValue = bigNumberValue.toNumber();

    return numberValue.toLocaleString(navigator.language, {
      style,
      currency,
      minimumFractionDigits,
      maximumFractionDigits,
    });
  } catch {
    // Fallback if toLocaleString fails
    return value.toString();
  }
}

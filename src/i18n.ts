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

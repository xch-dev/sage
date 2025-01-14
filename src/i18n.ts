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
  let numberValue: number;

  if (value instanceof BigNumber) {
    numberValue = value.toNumber();
  } else if (typeof value === 'string') {
    numberValue = parseFloat(value);
  } else {
    numberValue = value;
  }

  if (isNaN(numberValue)) return '';

  try {
    return numberValue.toLocaleString(navigator.language, {
      style,
      currency,
      minimumFractionDigits,
      maximumFractionDigits,
    });
  } catch (e) {
    // Fallback if toLocaleString fails
    return numberValue.toString();
  }
}

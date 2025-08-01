import BigNumber from 'bignumber.js';
import { NumberFormatProps, formatNumber } from '../i18n';

export function NumberFormat({
  value,
  style = 'decimal',
  currency,
  minimumFractionDigits,
  maximumFractionDigits,
}: NumberFormatProps) {
  try {
    const formatted = formatNumber({
      value,
      style,
      currency,
      minimumFractionDigits,
      maximumFractionDigits,
    });
    if (!formatted && formatted !== '0') return null;
    return <>{formatted}</>;
  } catch {
    // Fallback if formatting fails
    if (value instanceof BigNumber) {
      return <>{value.toFixed()}</>;
    }
    return <>{value?.toString() || ''}</>;
  }
}

import BigNumber from 'bignumber.js';

interface NumberFormatProps {
  value: string | number | BigNumber;
  style?: 'decimal' | 'currency' | 'percent';
  currency?: string;
  minimumFractionDigits?: number;
  maximumFractionDigits?: number;
}

export function NumberFormat({
  value,
  style = 'decimal',
  currency,
  minimumFractionDigits,
  maximumFractionDigits,
}: NumberFormatProps) {
  let numberValue: number;

  if (value instanceof BigNumber) {
    numberValue = value.toNumber();
  } else if (typeof value === 'string') {
    numberValue = parseFloat(value);
  } else {
    numberValue = value;
  }

  if (isNaN(numberValue)) return null;

  try {
    return (
      <>
        {numberValue.toLocaleString(navigator.language, {
          style,
          currency,
          minimumFractionDigits,
          maximumFractionDigits,
        })}
      </>
    );
  } catch (e) {
    // Fallback if toLocaleString fails
    return <>{numberValue.toString()}</>;
  }
}

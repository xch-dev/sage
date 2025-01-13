interface NumberFormatProps {
  value: string | number;
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
  const number = typeof value === 'string' ? parseFloat(value) : value;
  if (isNaN(number)) return null;

  return (
    <>
      {number.toLocaleString(navigator.language, {
        style,
        currency,
        minimumFractionDigits,
        maximumFractionDigits,
      })}
    </>
  );
}

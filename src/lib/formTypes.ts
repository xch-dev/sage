import BigNumber from 'bignumber.js';
import { z } from 'zod';

export const amount = (precision: number) =>
  z
    .string()
    .refine(
      (amount) => (BigNumber(amount).decimalPlaces() || 0) <= precision,
      'Value has too many decimals',
    )
    .refine((amount) => {
      const mojos = BigNumber(amount || 0).multipliedBy(
        BigNumber(10).pow(precision),
      );

      return mojos.isLessThanOrEqualTo(BigNumber('18446744073709551615'));
    }, 'Values is too large');

export const positiveAmount = (precision: number) =>
  amount(precision).refine(
    (amount) => BigNumber(amount).isGreaterThan(0),
    'Amount must be greater than 0',
  );

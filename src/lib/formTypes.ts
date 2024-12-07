import BigNumber from 'bignumber.js';
import { z } from 'zod';

export const amount = (decimals: number) =>
  z
    .string()
    .refine(
      (amount) => (BigNumber(amount).decimalPlaces() || 0) <= decimals,
      'Value has too many decimals',
    )
    .refine((amount) => {
      const mojos = BigNumber(amount || 0).multipliedBy(
        BigNumber(10).pow(decimals),
      );

      return mojos.isLessThanOrEqualTo(BigNumber('18446744073709551615'));
    }, 'Values is too large')

    .transform((amount) => {
      const mojos = BigNumber(amount || 0).multipliedBy(
        BigNumber(10).pow(decimals),
      );

      return mojos.toString();
    });

export const positiveAmount = (decimals: number) =>
  amount(decimals).refine(
    (amount) => BigNumber(amount).isGreaterThan(0),
    'Amount must be greater than 0',
  );

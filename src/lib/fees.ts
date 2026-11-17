import BigNumber from 'bignumber.js';
import { z } from 'zod';
import { commands, GetFeeRateResponse } from '../bindings';
import { amount } from './formTypes';
import { toMojos } from './utils';

/**
 * The value of a fee input when the wallet calculates the fee from the transaction's cost at
 * the recommended rate, rather than paying an amount the user entered.
 */
export const AUTO_FEE = 'auto';

const AUTO_FEE_PREFIX = `${AUTO_FEE}:`;

/**
 * The value of a fee input that calculates the fee at `rate` mojos per cost. An empty rate
 * means the recommended one.
 */
export function autoFeeAtRate(rate: string): string {
  return `${AUTO_FEE_PREFIX}${rate}`;
}

/** Whether a fee input's value calculates the fee, at the recommended or a chosen rate. */
export function isAutoFee(fee: string | number | null | undefined): boolean {
  return (
    typeof fee === 'string' &&
    (fee === AUTO_FEE || fee.startsWith(AUTO_FEE_PREFIX))
  );
}

/**
 * The rate an automatic fee was given, as it was entered, or `null` for the recommended rate.
 */
export function autoFeeRate(fee: string): string | null {
  return fee.startsWith(AUTO_FEE_PREFIX)
    ? fee.slice(AUTO_FEE_PREFIX.length)
    : null;
}

function isValidRate(rate: string): boolean {
  const value = BigNumber(rate);
  return value.isFinite() && value.isGreaterThanOrEqualTo(0);
}

/** A fee input's value that calculates the fee, with a valid rate if one was given. */
export const autoFee = z.string().refine((fee) => {
  if (!isAutoFee(fee)) return false;
  const rate = autoFeeRate(fee);
  return rate === null || rate === '' || isValidRate(rate);
}, 'Enter a valid fee rate');

/** A fee input's value: an automatic fee, or an amount in the network's unit. */
export const feeAmount = (precision: number) =>
  z.union([autoFee, amount(precision)]).optional();

let feeRates: Promise<GetFeeRateResponse> | null = null;

/**
 * The recommended and minimum fee rates, in mojos per cost. They're fetched once, and fetched
 * again if that fails.
 */
export function getFeeRates(): Promise<GetFeeRateResponse> {
  if (!feeRates) {
    feeRates = commands.getFeeRate({}).catch((error) => {
      feeRates = null;
      throw error;
    });
  }

  return feeRates;
}

export interface FeeParams {
  fee: string;
  fee_per_cost: number | null;
}

/**
 * The fee fields of a transaction request for a fee input's value. An automatic fee is paid
 * at its rate, or the recommended rate, and an entered amount is paid as is.
 */
export async function feeParams(
  fee: string | number | null | undefined,
  precision: number,
): Promise<FeeParams> {
  if (typeof fee === 'string' && isAutoFee(fee)) {
    const rate = autoFeeRate(fee);

    return {
      fee: '0',
      fee_per_cost:
        rate === null || rate === ''
          ? (await getFeeRates()).fee_per_cost
          : Number(rate),
    };
  }

  return {
    fee: toMojos(fee?.toString() || '0', precision),
    fee_per_cost: null,
  };
}

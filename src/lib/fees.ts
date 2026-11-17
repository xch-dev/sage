import { z } from 'zod';
import { commands } from '../bindings';
import { amount } from './formTypes';
import { toMojos } from './utils';

/**
 * The value of a fee input when the wallet calculates the fee from the transaction's cost,
 * rather than paying an amount the user entered.
 */
export const AUTO_FEE = 'auto';

export function isAutoFee(fee: string | number | null | undefined): boolean {
  return fee === AUTO_FEE;
}

/** A fee input's value: {@link AUTO_FEE}, or an amount in the network's unit. */
export const feeAmount = (precision: number) =>
  z.union([z.literal(AUTO_FEE), amount(precision)]).optional();

let feeRate: Promise<number> | null = null;

/**
 * The fee rate, in mojos per cost, that automatic fees are priced at. It's fetched once, and
 * fetched again if that fails.
 */
export function getFeeRate(): Promise<number> {
  if (!feeRate) {
    feeRate = commands
      .getFeeRate({})
      .then((response) => response.fee_per_cost)
      .catch((error) => {
        feeRate = null;
        throw error;
      });
  }

  return feeRate;
}

export interface FeeParams {
  fee: string;
  fee_per_cost: number | null;
}

/**
 * The fee fields of a transaction request for a fee input's value. An automatic fee is paid
 * at the wallet's fee rate, and an entered amount is paid as is.
 */
export async function feeParams(
  fee: string | number | null | undefined,
  precision: number,
): Promise<FeeParams> {
  if (isAutoFee(fee)) {
    return { fee: '0', fee_per_cost: await getFeeRate() };
  }

  return {
    fee: toMojos(fee?.toString() || '0', precision),
    fee_per_cost: null,
  };
}

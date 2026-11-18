import { t } from '@lingui/core/macro';
import BigNumber from 'bignumber.js';
import { z } from 'zod';
import { commands, FeeTierName, GetFeeRateResponse } from '../bindings';
import { amount } from './formTypes';
import { toMojos } from './utils';

/** The fee tiers, fastest first. Their rates come from {@link GetFeeRateResponse.tiers}. */
export const FEE_TIERS: readonly FeeTierName[] = [
  'fast',
  'standard',
  'economy',
];

/** The tier an automatic fee is at unless the user picks another one. */
export const DEFAULT_FEE_TIER: FeeTierName = 'standard';

const AUTO_FEE_PREFIX = 'auto:';

/**
 * The value of a fee input that calculates the fee from the transaction's cost, at the rate
 * estimated for `tier`. Without an estimate, it's the recommended rate.
 */
export function autoFeeAtTier(tier: FeeTierName): string {
  return `${AUTO_FEE_PREFIX}${tier}`;
}

/**
 * The value of a fee input when the wallet calculates the fee from the transaction's cost at
 * the default tier's rate, rather than paying an amount the user entered.
 */
export const AUTO_FEE = autoFeeAtTier(DEFAULT_FEE_TIER);

/**
 * What an automatic fee was stored as before it had tiers. It's read as the default tier.
 */
const LEGACY_AUTO_FEE = 'auto';

/**
 * The value of a fee input that calculates the fee at `rate` mojos per cost. An empty rate,
 * while one is being entered, means the recommended one.
 */
export function autoFeeAtRate(rate: string): string {
  return `${AUTO_FEE_PREFIX}${rate}`;
}

/** Whether a fee input's value calculates the fee, at a tier's rate or a chosen one. */
export function isAutoFee(
  fee: string | number | null | undefined,
): fee is string {
  return (
    typeof fee === 'string' &&
    (fee === LEGACY_AUTO_FEE || fee.startsWith(AUTO_FEE_PREFIX))
  );
}

/** A tier's name as it's shown. */
export function feeTierLabel(tier: FeeTierName): string {
  switch (tier) {
    case 'fast':
      return t`Fast`;
    case 'standard':
      return t`Standard`;
    case 'economy':
      return t`Economy`;
  }
}

/** How long a transaction paying a tier's estimated rate is expected to take to confirm. */
export function feeTierTime(tier: FeeTierName): string {
  switch (tier) {
    case 'fast':
      return t`About a minute`;
    case 'standard':
      return t`About 5 minutes`;
    case 'economy':
      return t`Within an hour`;
  }
}

/** A rate in mojos per cost as it's shown, to at most two decimal places. */
export function formatRate(rate: number): string {
  return BigNumber(rate).decimalPlaces(2, BigNumber.ROUND_UP).toString();
}

function isFeeTier(value: string): value is FeeTierName {
  return (FEE_TIERS as readonly string[]).includes(value);
}

/** The tier an automatic fee is at, or `null` if it's at a rate the user entered. */
export function autoFeeTier(fee: string): FeeTierName | null {
  if (fee === LEGACY_AUTO_FEE) return DEFAULT_FEE_TIER;
  const rest = fee.slice(AUTO_FEE_PREFIX.length);
  return isFeeTier(rest) ? rest : null;
}

/**
 * The rate an automatic fee was given, as it was entered, or `null` if it's at a tier. It's
 * empty while a rate is being entered.
 */
export function autoFeeRate(fee: string): string | null {
  if (autoFeeTier(fee) !== null) return null;
  return fee.slice(AUTO_FEE_PREFIX.length);
}

/**
 * The rate, in mojos per cost, estimated for `tier`, or the recommended rate if there's no
 * estimate, such as on a network without Coinset.
 */
export function tierFeeRate(
  rates: GetFeeRateResponse,
  tier: FeeTierName,
): number {
  return (
    rates.tiers.find((estimate) => estimate.name === tier)?.fee_per_cost ??
    rates.fee_per_cost
  );
}

/**
 * The rate, in mojos per cost, that an automatic fee is calculated at. A tier, or a rate that
 * hasn't been entered, needs the rates to be loaded, and is `null` until they are.
 */
export function autoFeeRateFor(
  fee: string,
  rates: GetFeeRateResponse | null,
): number | null {
  const tier = autoFeeTier(fee);
  if (tier !== null) return rates ? tierFeeRate(rates, tier) : null;

  const rate = autoFeeRate(fee);
  if (rate) return Number(rate);
  return rates ? rates.fee_per_cost : null;
}

function isValidRate(rate: string): boolean {
  const value = BigNumber(rate);
  return value.isFinite() && value.isGreaterThanOrEqualTo(0);
}

/** Whether an automatic fee is at a tier, or at a valid rate if it was given one. */
export function isValidAutoFee(fee: string): boolean {
  if (!isAutoFee(fee)) return false;
  const rate = autoFeeRate(fee);
  return !rate || isValidRate(rate);
}

/**
 * A fee input's value: an automatic fee with a valid rate, or a value `amountSchema` accepts.
 * The two are checked separately, rather than as a union, so that an invalid amount reports
 * its own message instead of a generic one.
 */
const feeSchema = (amountSchema: z.ZodType<string>) =>
  z.string().superRefine((fee, ctx) => {
    if (isAutoFee(fee)) {
      if (!isValidAutoFee(fee)) {
        ctx.addIssue({ code: 'custom', message: t`Enter a valid fee rate` });
      }
      return;
    }

    const result = amountSchema.safeParse(fee);

    if (!result.success) {
      for (const issue of result.error.issues) {
        ctx.addIssue({ code: 'custom', message: issue.message });
      }
    }
  });

/** A fee input's value: an automatic fee, or an amount in the network's unit. */
export const feeAmount = (precision: number) =>
  feeSchema(amount(precision)).optional();

/**
 * A required fee input's value: an automatic fee, or an amount in the network's unit that the
 * selectable balance covers.
 */
export const feeWithinBalance = (
  precision: number,
  selectableBalance: string | number,
) =>
  feeSchema(
    amount(precision).refine(
      (amount) => BigNumber(selectableBalance).gte(amount || 0),
      t`Not enough funds to cover the fee`,
    ),
  );

/**
 * How long fetched fee rates are used for before they're fetched again, since the tiers'
 * estimates follow the mempool.
 */
export const FEE_RATES_MAX_AGE_MS = 60_000;

let feeRates: {
  promise: Promise<GetFeeRateResponse>;
  fetchedAt: number;
} | null = null;

/**
 * The recommended and minimum fee rates, and the tiers' estimated rates, in mojos per cost.
 * They're fetched again once they're older than {@link FEE_RATES_MAX_AGE_MS}, or if fetching
 * them fails.
 */
export function getFeeRates(): Promise<GetFeeRateResponse> {
  if (!feeRates || Date.now() - feeRates.fetchedAt >= FEE_RATES_MAX_AGE_MS) {
    const promise = commands.getFeeRate({}).catch((error) => {
      if (feeRates?.promise === promise) feeRates = null;
      throw error;
    });

    feeRates = { promise, fetchedAt: Date.now() };
  }

  return feeRates.promise;
}

export interface FeeParams {
  fee: string;
  fee_per_cost: number | null;
}

/**
 * The fee fields of a transaction request for a fee input's value. An automatic fee is paid
 * at its tier's current rate or the rate entered for it, and an entered amount is paid as is.
 */
export async function feeParams(
  fee: string | number | null | undefined,
  precision: number,
): Promise<FeeParams> {
  if (isAutoFee(fee)) {
    const rate =
      autoFeeRateFor(fee, null) ?? autoFeeRateFor(fee, await getFeeRates());

    return { fee: '0', fee_per_cost: rate };
  }

  return {
    fee: toMojos(fee?.toString() || '0', precision),
    fee_per_cost: null,
  };
}

import { i18n } from '@lingui/core';
import { beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';

const getFeeRate = vi.hoisted(() => vi.fn());

vi.mock('../bindings', () => ({ commands: { getFeeRate } }));

import {
  AUTO_FEE,
  autoFeeAtRate,
  autoFeeRate,
  feeAmount,
  feeParams,
  feeWithinBalance,
  isAutoFee,
  isValidAutoFee,
} from './fees';

beforeAll(() => {
  i18n.loadAndActivate({ locale: 'en', messages: {} });
});

describe('automatic fees', () => {
  it('tells automatic fees from amounts', () => {
    expect(isAutoFee(AUTO_FEE)).toBe(true);
    expect(isAutoFee(autoFeeAtRate('7.5'))).toBe(true);
    expect(isAutoFee('0.001')).toBe(false);
    expect(isAutoFee('')).toBe(false);
    expect(isAutoFee(undefined)).toBe(false);
  });

  it('reads the rate an automatic fee was given', () => {
    expect(autoFeeRate(AUTO_FEE)).toBeNull();
    expect(autoFeeRate(autoFeeAtRate('7.5'))).toBe('7.5');
    expect(autoFeeRate(autoFeeAtRate(''))).toBeNull();
  });

  it('accepts a missing or valid rate, and rejects anything else', () => {
    expect(isValidAutoFee(AUTO_FEE)).toBe(true);
    expect(isValidAutoFee(autoFeeAtRate(''))).toBe(true);
    expect(isValidAutoFee(autoFeeAtRate('2'))).toBe(true);
    expect(isValidAutoFee(autoFeeAtRate('abc'))).toBe(false);
    expect(isValidAutoFee(autoFeeAtRate('-1'))).toBe(false);
    expect(feeAmount(12).safeParse(autoFeeAtRate('abc')).success).toBe(false);
    expect(feeAmount(12).safeParse('0.001').success).toBe(true);
  });

  it('builds request fields for a chosen rate or an amount', async () => {
    await expect(feeParams(autoFeeAtRate('7.5'), 12)).resolves.toEqual({
      fee: '0',
      fee_per_cost: 7.5,
    });
    await expect(feeParams('0.001', 12)).resolves.toEqual({
      fee: '1000000000',
      fee_per_cost: null,
    });
    await expect(feeParams(undefined, 12)).resolves.toEqual({
      fee: '0',
      fee_per_cost: null,
    });
  });
});

describe('the recommended rate', () => {
  const rates = { fee_per_cost: 5, minimum_fee_per_cost: 5 };

  beforeEach(() => {
    // The rates are cached by the module, so each test starts from a fresh copy of it.
    vi.resetModules();
    getFeeRate.mockReset();
  });

  it('prices an automatic fee at the recommended rate', async () => {
    getFeeRate.mockResolvedValue(rates);
    const fees = await import('./fees');

    await expect(fees.feeParams(fees.AUTO_FEE, 12)).resolves.toEqual({
      fee: '0',
      fee_per_cost: 5,
    });
    await expect(fees.feeParams(fees.autoFeeAtRate(''), 12)).resolves.toEqual({
      fee: '0',
      fee_per_cost: 5,
    });
    expect(getFeeRate).toHaveBeenCalledTimes(1);
  });

  it('fetches the rates again after a failure', async () => {
    getFeeRate
      .mockRejectedValueOnce(new Error('offline'))
      .mockResolvedValueOnce(rates);
    const fees = await import('./fees');

    await expect(fees.getFeeRates()).rejects.toThrow('offline');
    await expect(fees.getFeeRates()).resolves.toEqual(rates);
    expect(getFeeRate).toHaveBeenCalledTimes(2);
  });
});

describe('fee validation', () => {
  it('reports why an entered amount is invalid', () => {
    const schema = feeWithinBalance(12, '1');

    expect(schema.safeParse(AUTO_FEE).success).toBe(true);
    expect(schema.safeParse('0.5').success).toBe(true);

    const tooMuch = schema.safeParse('5');
    expect(tooMuch.success).toBe(false);
    expect(tooMuch.error?.issues.map((issue) => issue.message)).toEqual([
      'Not enough funds to cover the fee',
    ]);
  });
});

import { i18n } from '@lingui/core';
import {
  afterEach,
  beforeAll,
  beforeEach,
  describe,
  expect,
  it,
  vi,
} from 'vitest';

const getFeeRate = vi.hoisted(() => vi.fn());

vi.mock('../bindings', () => ({ commands: { getFeeRate } }));

import {
  AUTO_FEE,
  autoFeeAtRate,
  autoFeeAtTier,
  autoFeeRate,
  autoFeeRateFor,
  autoFeeTier,
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
    expect(autoFeeRate(autoFeeAtRate(''))).toBe('');
  });

  it('reads the tier an automatic fee is at', () => {
    expect(AUTO_FEE).toBe(autoFeeAtTier('standard'));
    expect(autoFeeTier(autoFeeAtTier('fast'))).toBe('fast');
    expect(autoFeeTier(autoFeeAtTier('economy'))).toBe('economy');
    expect(autoFeeTier(autoFeeAtRate('7.5'))).toBeNull();
    expect(autoFeeTier(autoFeeAtRate(''))).toBeNull();
    // Stored before there were tiers.
    expect(isAutoFee('auto')).toBe(true);
    expect(autoFeeTier('auto')).toBe('standard');
  });

  it('prices a tier at its estimate, or the recommended rate without one', () => {
    const rates = {
      fee_per_cost: 5,
      minimum_fee_per_cost: 5,
      tiers: [
        { name: 'fast' as const, target_time: 60, fee_per_cost: 12.5 },
        { name: 'standard' as const, target_time: 300, fee_per_cost: 8 },
        { name: 'economy' as const, target_time: 3600, fee_per_cost: 5 },
      ],
    };
    const unestimated = { ...rates, tiers: [] };

    expect(autoFeeRateFor(autoFeeAtTier('fast'), rates)).toBe(12.5);
    expect(autoFeeRateFor(autoFeeAtTier('standard'), rates)).toBe(8);
    expect(autoFeeRateFor(autoFeeAtTier('fast'), unestimated)).toBe(5);
    expect(autoFeeRateFor(autoFeeAtTier('fast'), null)).toBeNull();
    expect(autoFeeRateFor(autoFeeAtRate('7.5'), null)).toBe(7.5);
    expect(autoFeeRateFor(autoFeeAtRate(''), rates)).toBe(5);
  });

  it('accepts a missing or valid rate, and rejects anything else', () => {
    expect(isValidAutoFee(AUTO_FEE)).toBe(true);
    expect(isValidAutoFee(autoFeeAtTier('fast'))).toBe(true);
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
  const rates = {
    fee_per_cost: 5,
    minimum_fee_per_cost: 5,
    tiers: [
      { name: 'fast', target_time: 60, fee_per_cost: 12.5 },
      { name: 'standard', target_time: 300, fee_per_cost: 8 },
      { name: 'economy', target_time: 3600, fee_per_cost: 5 },
    ],
  };

  beforeEach(() => {
    // The rates are cached by the module, so each test starts from a fresh copy of it.
    vi.resetModules();
    getFeeRate.mockReset();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("prices an automatic fee at its tier's rate", async () => {
    getFeeRate.mockResolvedValue(rates);
    const fees = await import('./fees');

    await expect(fees.feeParams(fees.AUTO_FEE, 12)).resolves.toEqual({
      fee: '0',
      fee_per_cost: 8,
    });
    await expect(
      fees.feeParams(fees.autoFeeAtTier('fast'), 12),
    ).resolves.toEqual({ fee: '0', fee_per_cost: 12.5 });
    await expect(fees.feeParams(fees.autoFeeAtRate(''), 12)).resolves.toEqual({
      fee: '0',
      fee_per_cost: 5,
    });
    expect(getFeeRate).toHaveBeenCalledTimes(1);
  });

  it("doesn't fetch the rates for a rate that was entered", async () => {
    const fees = await import('./fees');

    await expect(fees.feeParams(fees.autoFeeAtRate('7'), 12)).resolves.toEqual({
      fee: '0',
      fee_per_cost: 7,
    });
    expect(getFeeRate).not.toHaveBeenCalled();
  });

  it('fetches the rates again once they are stale', async () => {
    vi.useFakeTimers();
    getFeeRate.mockResolvedValue(rates);
    const fees = await import('./fees');

    await fees.getFeeRates();
    vi.advanceTimersByTime(fees.FEE_RATES_MAX_AGE_MS - 1);
    await fees.getFeeRates();
    expect(getFeeRate).toHaveBeenCalledTimes(1);

    vi.advanceTimersByTime(1);
    await fees.getFeeRates();
    expect(getFeeRate).toHaveBeenCalledTimes(2);
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

  it('keeps newer rates when a stale request fails', async () => {
    vi.useFakeTimers();
    let failStale: (error: Error) => void = () => undefined;
    getFeeRate
      .mockReturnValueOnce(
        new Promise((_, reject) => {
          failStale = reject;
        }),
      )
      .mockResolvedValueOnce(rates);
    const fees = await import('./fees');

    const stale = fees.getFeeRates();
    vi.advanceTimersByTime(fees.FEE_RATES_MAX_AGE_MS);
    await expect(fees.getFeeRates()).resolves.toEqual(rates);

    failStale(new Error('offline'));
    await expect(stale).rejects.toThrow('offline');

    // The newer rates are still cached, so they aren't fetched a third time.
    await expect(fees.getFeeRates()).resolves.toEqual(rates);
    expect(getFeeRate).toHaveBeenCalledTimes(2);
  });

  it('formats a rate to at most two decimal places, rounding up', async () => {
    const fees = await import('./fees');

    expect(fees.formatRate(5)).toBe('5');
    expect(fees.formatRate(48.15)).toBe('48.15');
    expect(fees.formatRate(5.001)).toBe('5.01');
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

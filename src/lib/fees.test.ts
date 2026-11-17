import { describe, expect, it } from 'vitest';
import {
  AUTO_FEE,
  autoFee,
  autoFeeAtRate,
  autoFeeRate,
  feeAmount,
  feeParams,
  isAutoFee,
} from './fees';

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

  it('accepts a missing or valid rate, and rejects anything else', () => {
    expect(autoFee.safeParse(AUTO_FEE).success).toBe(true);
    expect(autoFee.safeParse(autoFeeAtRate('')).success).toBe(true);
    expect(autoFee.safeParse(autoFeeAtRate('2')).success).toBe(true);
    expect(autoFee.safeParse(autoFeeAtRate('abc')).success).toBe(false);
    expect(autoFee.safeParse(autoFeeAtRate('-1')).success).toBe(false);
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

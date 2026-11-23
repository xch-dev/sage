// @vitest-environment jsdom

import { act, renderHook } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const getFeeRates = vi.hoisted(() => vi.fn());

vi.mock('@/lib/fees', () => ({ FEE_RATES_MAX_AGE_MS: 60_000, getFeeRates }));

import { useFeeRates } from './useFeeRates';

const rates = (fee_per_cost: number) => ({
  fee_per_cost,
  minimum_fee_per_cost: 5,
  tiers: [],
});

/** Lets the hook's pending request settle. */
async function settle() {
  await act(async () => {
    await Promise.resolve();
  });
}

describe('useFeeRates', () => {
  beforeEach(() => {
    vi.useFakeTimers();
    getFeeRates.mockReset();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('loads the rates, and reloads them while it is mounted', async () => {
    getFeeRates.mockResolvedValueOnce(rates(5)).mockResolvedValueOnce(rates(8));
    const { result } = renderHook(() => useFeeRates());

    expect(result.current).toBeNull();
    await settle();
    expect(result.current).toEqual(rates(5));

    act(() => vi.advanceTimersByTime(30_000));
    await settle();
    expect(result.current).toEqual(rates(8));
  });

  it('keeps the last rates when reloading them fails', async () => {
    getFeeRates
      .mockResolvedValueOnce(rates(5))
      .mockRejectedValueOnce(new Error('offline'));
    const { result } = renderHook(() => useFeeRates());
    await settle();

    act(() => vi.advanceTimersByTime(30_000));
    await settle();
    expect(getFeeRates).toHaveBeenCalledTimes(2);
    expect(result.current).toEqual(rates(5));
  });

  it('stops reloading them once it is unmounted', async () => {
    getFeeRates.mockResolvedValue(rates(5));
    const { unmount } = renderHook(() => useFeeRates());
    await settle();

    unmount();
    act(() => vi.advanceTimersByTime(120_000));
    expect(getFeeRates).toHaveBeenCalledTimes(1);
  });
});

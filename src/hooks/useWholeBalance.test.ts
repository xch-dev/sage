// @vitest-environment jsdom

import { act, cleanup, renderHook } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import type { TokenRecord } from '@/bindings';
import { useWholeBalance } from './useWholeBalance';

afterEach(() => {
  cleanup();
});

function xch(balance: string): TokenRecord {
  return {
    asset_id: null,
    precision: 12,
    selectable_balance: balance,
  } as unknown as TokenRecord;
}

function render(asset: TokenRecord | null, enabled = true) {
  const setAmount = vi.fn();
  const hook = renderHook(
    (props: { asset: TokenRecord | null; enabled: boolean }) =>
      useWholeBalance({ ...props, setAmount }),
    { initialProps: { asset, enabled } },
  );
  return { ...hook, setAmount };
}

describe('useWholeBalance', () => {
  it('spends the whole balance once switched on', () => {
    const { result, setAmount } = render(xch('5000000000000'));
    expect(result.current.active).toBe(false);
    expect(result.current.mojos).toBeNull();

    act(() => result.current.onWholeBalanceChange?.(true));

    expect(result.current.active).toBe(true);
    expect(result.current.mojos).toBe('5000000000000');
    expect(setAmount).toHaveBeenLastCalledWith('5');
  });

  it('follows the balance as it changes', () => {
    const { result, rerender, setAmount } = render(xch('5000000000000'));
    act(() => result.current.onWholeBalanceChange?.(true));

    rerender({ asset: xch('4000000000000'), enabled: true });

    expect(result.current.mojos).toBe('4000000000000');
    expect(setAmount).toHaveBeenLastCalledWith('4');
  });

  it('clears the amount when switched off', () => {
    const { result, setAmount } = render(xch('5000000000000'));
    act(() => result.current.onWholeBalanceChange?.(true));
    act(() => result.current.onWholeBalanceChange?.(false));

    expect(result.current.active).toBe(false);
    expect(result.current.mojos).toBeNull();
    expect(setAmount).toHaveBeenLastCalledWith('');
  });

  it("can't be switched on without a balance, or when it isn't allowed", () => {
    expect(
      render(xch('0')).result.current.onWholeBalanceChange,
    ).toBeUndefined();
    expect(render(null).result.current.onWholeBalanceChange).toBeUndefined();
    expect(
      render(xch('5000000000000'), false).result.current.onWholeBalanceChange,
    ).toBeUndefined();
  });

  it("stops for good when it's no longer possible", () => {
    const { result, rerender, setAmount } = render(xch('5000000000000'));
    act(() => result.current.onWholeBalanceChange?.(true));

    // The balance runs out, so it stops and the amount is cleared.
    rerender({ asset: xch('0'), enabled: true });
    expect(result.current.active).toBe(false);
    expect(setAmount).toHaveBeenLastCalledWith('');

    // It doesn't come back by itself when the balance does.
    rerender({ asset: xch('3000000000000'), enabled: true });
    expect(result.current.active).toBe(false);
    expect(result.current.mojos).toBeNull();
  });

  it("stops when switching to something that can't be the whole balance", () => {
    const { result, rerender } = render(xch('5000000000000'));
    act(() => result.current.onWholeBalanceChange?.(true));

    // Like turning on a bulk send.
    rerender({ asset: xch('5000000000000'), enabled: false });
    expect(result.current.active).toBe(false);

    rerender({ asset: xch('5000000000000'), enabled: true });
    expect(result.current.active).toBe(false);
  });
});

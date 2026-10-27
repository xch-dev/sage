// @vitest-environment jsdom

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const { getHandles } = vi.hoisted(() => ({ getHandles: vi.fn() }));

vi.mock('@/bindings', () => ({ commands: { getHandles } }));

import { addressHandles, refreshOwnedHandles } from './addressHandles';

const HOUR = 60 * 60 * 1000;

describe('refreshing owned handles', () => {
  beforeEach(() => {
    vi.useFakeTimers({ now: 1_000 * HOUR, toFake: ['Date'] });
    localStorage.clear();
    getHandles.mockReset();
    getHandles.mockResolvedValue({ handles: [], total: 0 });
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('verifies owned handles through the backend refresh', async () => {
    await refreshOwnedHandles(1, 'txch');

    expect(getHandles).toHaveBeenCalledWith({
      filter: 'owned',
      include_unverified: false,
      refresh: true,
      offset: 0,
      limit: 1,
    });
  });

  it('runs at most every 12 hours for each wallet and network', async () => {
    await refreshOwnedHandles(1, 'txch');
    vi.advanceTimersByTime(11 * HOUR);
    await refreshOwnedHandles(1, 'txch');
    expect(getHandles).toHaveBeenCalledTimes(1);

    // Another wallet, or the same wallet on another network, isn't held back.
    await refreshOwnedHandles(2, 'txch');
    await refreshOwnedHandles(1, 'xch');
    expect(getHandles).toHaveBeenCalledTimes(3);

    vi.advanceTimersByTime(HOUR + 1);
    await refreshOwnedHandles(1, 'txch');
    expect(getHandles).toHaveBeenCalledTimes(4);
  });

  it('tries again next time if the refresh fails', async () => {
    getHandles.mockRejectedValueOnce({ kind: 'api', reason: 'unavailable' });

    await expect(refreshOwnedHandles(1, 'txch')).rejects.toBeTruthy();
    await refreshOwnedHandles(1, 'txch');

    expect(getHandles).toHaveBeenCalledTimes(2);
  });

  it('relabels shown addresses once owned handles are verified', async () => {
    const refresh = vi.spyOn(addressHandles, 'refresh');

    await refreshOwnedHandles(1, 'txch');

    expect(refresh).toHaveBeenCalledTimes(1);
    refresh.mockRestore();
  });

  it("doesn't try again on networks without XCHandles", async () => {
    getHandles.mockRejectedValueOnce({
      kind: 'api',
      reason: 'unsupported',
      handle_error_code: 'unsupported_network',
    });

    await refreshOwnedHandles(1, 'txch');
    await refreshOwnedHandles(1, 'txch');

    expect(getHandles).toHaveBeenCalledTimes(1);
  });
});

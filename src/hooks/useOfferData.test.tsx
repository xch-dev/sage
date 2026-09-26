// @vitest-environment jsdom

import { act, renderHook, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { OfferParams } from './useOfferParams';

const getOffers = vi.fn();

vi.mock('@/bindings', () => ({
  commands: { getOffers: (...args: unknown[]) => getOffers(...args) },
  events: { syncEvent: { listen: () => Promise.resolve(() => undefined) } },
}));

// Stable across renders, like the real hook; a fresh fn per render would
// recreate `refresh` every render and refetch in a loop.
const addError = vi.fn();

vi.mock('@/hooks/useErrors', () => ({
  useErrors: () => ({ addError }),
}));

const { useOfferData } = await import('./useOfferData');

const base: OfferParams = {
  page: 1,
  pageSize: 2,
  query: null,
  status: 'all',
  findSide: 'any',
  sort: 'created',
  ascending: false,
};

function offer(id: string) {
  return { offer_id: id } as never;
}

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (error: unknown) => void;
  const promise = new Promise<T>((res, rej) => {
    resolve = res;
    reject = rej;
  });
  // Prevent unhandled-rejection noise when a stale promise is rejected
  // after the test has already made its assertions.
  promise.catch(() => undefined);
  return { promise, resolve, reject };
}

beforeEach(() => {
  getOffers.mockReset();
  addError.mockReset();
});

describe('useOfferData', () => {
  it('sends paging, filter, and sort fields', async () => {
    getOffers.mockResolvedValue({ offers: [], total: 0 });
    renderHook(() =>
      useOfferData({
        ...base,
        page: 3,
        status: 'active',
        query: 'sbx',
        findSide: 'offered',
      }),
    );
    await waitFor(() => expect(getOffers).toHaveBeenCalled());
    expect(getOffers).toHaveBeenLastCalledWith({
      offset: 4,
      limit: 2,
      status: 'active',
      find_value: 'sbx',
      find_side: 'offered',
      sort_mode: 'created',
      ascending: false,
    });
  });

  it('sends null status for "all"', async () => {
    getOffers.mockResolvedValue({ offers: [], total: 0 });
    renderHook(() => useOfferData(base));
    await waitFor(() => expect(getOffers).toHaveBeenCalled());
    expect(getOffers.mock.calls[0][0].status).toBeNull();
  });

  it('ignores stale responses', async () => {
    const first = deferred<{ offers: unknown[]; total: number }>();
    const second = deferred<{ offers: unknown[]; total: number }>();
    getOffers
      .mockReturnValueOnce(first.promise)
      .mockReturnValueOnce(second.promise);

    const { result, rerender } = renderHook(
      (p: OfferParams) => useOfferData(p),
      {
        initialProps: base,
      },
    );
    rerender({ ...base, page: 2 });

    await act(async () =>
      second.resolve({ offers: [offer('page2')], total: 3 }),
    );
    await act(async () =>
      first.resolve({ offers: [offer('page1')], total: 3 }),
    );

    expect(result.current.offers.map((o) => o.offer_id)).toEqual(['page2']);
  });

  it('flags past-end pages', async () => {
    getOffers.mockResolvedValue({ offers: [], total: 0 });
    const { result } = renderHook(() => useOfferData({ ...base, page: 3 }));
    expect(result.current.isPastEnd).toBe(false);
    await waitFor(() => expect(result.current.loaded).toBe(true));
    expect(result.current.isPastEnd).toBe(true);
  });

  it('does not flag an empty first page as past-end', async () => {
    getOffers.mockResolvedValue({ offers: [], total: 0 });
    const { result } = renderHook(() => useOfferData(base));
    await waitFor(() => expect(result.current.loaded).toBe(true));
    expect(result.current.isPastEnd).toBe(false);
  });

  it('reports an error from the current request', async () => {
    const error = new Error('boom');
    getOffers.mockRejectedValue(error);
    const { result } = renderHook(() => useOfferData(base));
    await waitFor(() => expect(addError).toHaveBeenCalledTimes(1));
    expect(addError).toHaveBeenCalledWith(error);
    expect(result.current.isLoading).toBe(false);
    expect(result.current.loaded).toBe(false);
  });

  it('is not current until a response for the active params has loaded', async () => {
    const first = deferred<{ offers: unknown[]; total: number }>();
    getOffers.mockReturnValueOnce(first.promise);

    const { result } = renderHook(() => useOfferData(base));
    expect(result.current.isCurrent).toBe(false);

    await act(async () => first.resolve({ offers: [], total: 0 }));
    expect(result.current.isCurrent).toBe(true);
  });

  it('flags stale data as not current while params have changed but the new response has not arrived', async () => {
    const first = deferred<{ offers: unknown[]; total: number }>();
    const second = deferred<{ offers: unknown[]; total: number }>();
    getOffers
      .mockReturnValueOnce(first.promise)
      .mockReturnValueOnce(second.promise);

    const { result, rerender } = renderHook(
      (p: OfferParams) => useOfferData(p),
      { initialProps: base },
    );

    await act(async () => first.resolve({ offers: [], total: 0 }));
    expect(result.current.isCurrent).toBe(true);

    // Simulates clicking "Clear filters": params change (e.g. status), but
    // the refreshed data for the new params hasn't arrived yet. The old,
    // now-mismatched response must not be reported as current, so the UI
    // doesn't flash an intro/no-matches state derived from stale params.
    rerender({ ...base, status: 'active' });
    expect(result.current.isCurrent).toBe(false);

    await act(async () => second.resolve({ offers: [], total: 0 }));
    expect(result.current.isCurrent).toBe(true);
  });

  it('ignores an error from a superseded request', async () => {
    const first = deferred<{ offers: unknown[]; total: number }>();
    const second = deferred<{ offers: unknown[]; total: number }>();
    getOffers
      .mockReturnValueOnce(first.promise)
      .mockReturnValueOnce(second.promise);

    const { result, rerender } = renderHook(
      (p: OfferParams) => useOfferData(p),
      {
        initialProps: base,
      },
    );
    rerender({ ...base, page: 2 });

    await act(async () =>
      second.resolve({ offers: [offer('page2')], total: 3 }),
    );
    await act(async () => first.reject(new Error('stale failure')));

    expect(addError).not.toHaveBeenCalled();
    expect(result.current.offers.map((o) => o.offer_id)).toEqual(['page2']);
  });
});

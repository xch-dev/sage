// @vitest-environment jsdom

import { usePrices } from '@/hooks/usePrices';
import { cleanup, renderHook, waitFor } from '@testing-library/react';
import { ReactNode } from 'react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { PriceProvider } from './PriceContext';

vi.mock('@/hooks/useNetwork', () => ({
  useNetwork: () => ({ network: 'mainnet', isTestnet: false }),
}));

const fetchMock = vi.fn();

function respond(url: string) {
  const body = url.includes('coingecko')
    ? { chia: { usd: 20, eur: 18 } }
    : { tickers: [] };
  return Promise.resolve(new Response(JSON.stringify(body)));
}

function wrapper({ children }: { children: ReactNode }) {
  return <PriceProvider>{children}</PriceProvider>;
}

function coinGeckoUrls() {
  return fetchMock.mock.calls
    .map(([url]) => String(url))
    .filter((url) => url.includes('coingecko'));
}

beforeEach(() => {
  fetchMock.mockImplementation(respond);
  vi.stubGlobal('fetch', fetchMock);
});

afterEach(() => {
  cleanup();
  window.localStorage.clear();
  fetchMock.mockReset();
  vi.unstubAllGlobals();
});

describe('PriceProvider', () => {
  it('looks up the XCH price in the chosen currency', async () => {
    window.localStorage.setItem('sage-fiat-currency', '"EUR"');

    const { result } = renderHook(() => usePrices(), { wrapper });

    expect(result.current.fiatCurrency).toBe('EUR');
    await waitFor(() => expect(result.current.getPriceInFiat(null)).toBe(18));
    expect(coinGeckoUrls()).toEqual([
      expect.stringContaining('vs_currencies=eur'),
    ]);
  });

  it('falls back to USD for an unsupported stored currency', async () => {
    window.localStorage.setItem('sage-fiat-currency', '"XYZ"');

    const { result } = renderHook(() => usePrices(), { wrapper });

    expect(result.current.fiatCurrency).toBe('USD');
    await waitFor(() => expect(result.current.getPriceInFiat(null)).toBe(20));
    expect(coinGeckoUrls()).toEqual([
      expect.stringContaining('vs_currencies=usd'),
    ]);
  });

  it('makes no requests while price lookups are off', async () => {
    window.localStorage.setItem('sage-price-lookups', 'false');

    const { result } = renderHook(() => usePrices(), { wrapper });

    expect(result.current.fiatCurrency).toBeNull();
    // Give an effect that would fetch the chance to run.
    await new Promise((resolve) => setTimeout(resolve, 10));
    expect(fetchMock).not.toHaveBeenCalled();
    expect(result.current.getPriceInFiat(null)).toBe(0);
  });
});

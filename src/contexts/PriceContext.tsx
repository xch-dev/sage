import { useNetwork } from '@/hooks/useNetwork';
import { usePriceSources } from '@/hooks/usePriceSources';
import {
  ActivePriceSource,
  activePriceSources,
  firstCatPriceXch,
  firstLabel,
  firstXchUsd,
  indexLayer,
  PriceLayer,
  RefreshPlan,
  refreshPlan,
} from '@/lib/priceSources';
import {
  isIndexForNetwork,
  parseTokenIndex,
  TokenIndex,
  TokenIndexEntry,
} from '@/lib/tokenIndex';
import { dexieApiUrl } from '@/lib/urls';
import {
  createContext,
  ReactNode,
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
} from 'react';

// Add an interface for the price data structure
interface CatPriceData {
  lastPrice: number | null;
  askPrice: number | null;
}

interface DexieTicker {
  base_currency: string;
  base_name: string;
  base_code: string;
  last_price: string | null;
  ask: string | null;
}

interface DexieResponse {
  tickers: DexieTicker[];
}

export interface PriceContextType {
  getBalanceInUsd: (assetId: string | null, balance: string) => string;
  getPriceInUsd: (assetId: string | null) => number;
  getCatAskPriceInXch: (assetId: string) => number | null;
  /** A display-only label from a token index, for a CAT with no name. */
  getSuggestedLabel: (assetId: string | null) => TokenIndexEntry | null;
  /** Which source Refresh Info reads for a CAT, by the user's order. */
  getRefreshPlan: (assetId: string) => RefreshPlan;
  isLoading: boolean;
}

export const PriceContext = createContext<PriceContextType | undefined>(
  undefined,
);

export function PriceProvider({ children }: { children: ReactNode }) {
  const [xchUsdPrice, setXchUsdPrice] = useState<number>(0);
  const [catPrices, setCatPrices] = useState<Record<string, CatPriceData>>({});
  const [indexes, setIndexes] = useState<Record<string, TokenIndex>>({});
  const { network, isTestnet } = useNetwork();
  const [isPriceLoading, setIsPriceLoading] = useState(false);
  const intervalRef = useRef<NodeJS.Timeout | null>(null);
  const { sources } = usePriceSources();

  const active = useMemo(
    () => activePriceSources(sources, isTestnet),
    [sources, isTestnet],
  );

  // What to fetch, independent of order: reordering sources needs no refetch.
  const fetchKey = JSON.stringify(
    [...active].sort((a, b) => a.id.localeCompare(b.id)),
  );

  // Indexes are per network.
  useEffect(() => {
    setIndexes({});
  }, [network, isTestnet]);

  useEffect(() => {
    // Don't fetch prices until network is loaded
    if (network === null) {
      return;
    }

    if (intervalRef.current) {
      clearInterval(intervalRef.current);
      intervalRef.current = null;
    }

    const targets: ActivePriceSource[] = JSON.parse(fetchKey);
    const useDexie = targets.some((source) => source.kind === 'dexie');
    const useCoinGecko = targets.some((source) => source.kind === 'coingecko');
    const indexTargets = targets.filter(
      (source) => source.kind === 'index' && source.url,
    );

    // Forget indexes whose source was turned off or removed.
    setIndexes((previous) =>
      Object.fromEntries(
        Object.entries(previous).filter(([id]) =>
          indexTargets.some((source) => source.id === id),
        ),
      ),
    );

    if (!useDexie) setCatPrices({});

    const fetchCatPrices = async () => {
      if (!useDexie) return;

      try {
        const response = await fetch(
          dexieApiUrl('v3/prices/tickers', isTestnet),
        );

        if (!response.ok) {
          throw new Error(`HTTP error! status: ${response.status}`);
        }

        const data: DexieResponse = await response.json();
        const tickers = data.tickers.reduce(
          (acc: Record<string, CatPriceData>, ticker: DexieTicker) => {
            acc[ticker.base_currency.toLowerCase()] = {
              lastPrice: ticker.last_price ? Number(ticker.last_price) : null,
              askPrice: ticker.ask ? Number(ticker.ask) : null,
            };
            return acc;
          },
          {},
        );
        setCatPrices(tickers);
      } catch (error) {
        console.error('Failed to fetch CAT prices:', error);
        setCatPrices({});
      }
    };

    const fetchChiaPrice = async () => {
      if (!useCoinGecko) return;

      try {
        const response = await fetch(
          'https://api.coingecko.com/api/v3/simple/price?ids=chia&vs_currencies=usd',
        );

        if (!response.ok) {
          throw new Error(`HTTP error! status: ${response.status}`);
        }

        const data = await response.json();
        const newPrice = data.chia?.usd;

        if (newPrice && newPrice >= 0) {
          setXchUsdPrice(newPrice);
        } else {
          console.warn('Invalid XCH price received:', newPrice);
          // Don't update the price if it's invalid, keep the previous value
        }
      } catch (error) {
        console.error('Failed to fetch Chia price:', error);
        // Don't set to 0, keep the previous value
      }
    };

    // Token indexes added in Settings. A failed fetch keeps the last good one.
    const fetchTokenIndex = async (source: ActivePriceSource) => {
      if (!source.url) return;

      try {
        const response = await fetch(source.url);

        if (!response.ok) {
          throw new Error(`HTTP error! status: ${response.status}`);
        }

        const index = parseTokenIndex(await response.json());

        if (!index) {
          throw new Error('Unsupported token index format');
        }

        if (!isIndexForNetwork(index, isTestnet)) {
          throw new Error(`Token index is for ${index.network || 'unknown'}`);
        }

        setIndexes((previous) => ({ ...previous, [source.id]: index }));
      } catch (error) {
        console.warn(`Failed to fetch token index ${source.url}:`, error);
      }
    };

    const fetchPrices = async () => {
      setIsPriceLoading(true);
      try {
        await Promise.all([
          fetchCatPrices(),
          fetchChiaPrice(),
          ...indexTargets.map(fetchTokenIndex),
        ]);
      } finally {
        setIsPriceLoading(false);
      }
    };

    fetchPrices();
    intervalRef.current = setInterval(fetchPrices, 60000);

    return () => {
      if (intervalRef.current) {
        clearInterval(intervalRef.current);
        intervalRef.current = null;
      }
    };
  }, [network, isTestnet, fetchKey]);

  // What each enabled source knows, in the user's order of priority.
  const layers = useMemo<PriceLayer[]>(
    () =>
      active.map((source) => {
        switch (source.kind) {
          case 'dexie':
            return {
              catPricesXch: Object.fromEntries(
                Object.entries(catPrices).map(([id, data]) => [
                  id,
                  data.lastPrice,
                ]),
              ),
            };
          case 'coingecko':
            return { xchUsd: xchUsdPrice };
          case 'index':
            return indexLayer(indexes[source.id], isTestnet);
        }
      }),
    [active, catPrices, xchUsdPrice, indexes, isTestnet],
  );

  const usdPerXch = useMemo(() => firstXchUsd(layers), [layers]);

  const getPriceInUsd = useCallback(
    (assetId: string | null) => {
      if (!assetId) return usdPerXch;

      const xchPrice = firstCatPriceXch(layers, assetId);

      if (xchPrice === null) {
        return 0;
      }

      return xchPrice * usdPerXch;
    },
    [usdPerXch, layers],
  );

  const getBalanceInUsd = useCallback(
    (assetId: string | null, balance: string) => {
      // Validate balance input
      const balanceNum = Number(balance);
      if (isNaN(balanceNum)) {
        return '0.00';
      }

      if (!assetId) return (balanceNum * usdPerXch).toFixed(2);

      const xchPrice = firstCatPriceXch(layers, assetId);

      if (xchPrice === null) {
        return '0.00';
      }

      return (balanceNum * xchPrice * usdPerXch).toFixed(2);
    },
    [usdPerXch, layers],
  );

  const getCatAskPriceInXch = useCallback(
    (assetId: string) => {
      const priceData = catPrices[assetId.toLowerCase()];
      return priceData?.askPrice ?? null;
    },
    [catPrices],
  );

  const getSuggestedLabel = useCallback(
    (assetId: string | null) => (assetId ? firstLabel(layers, assetId) : null),
    [layers],
  );

  const getRefreshPlan = useCallback(
    (assetId: string) => refreshPlan(active, layers, assetId),
    [active, layers],
  );

  return (
    <PriceContext.Provider
      value={{
        getBalanceInUsd,
        getPriceInUsd,
        getCatAskPriceInXch,
        getSuggestedLabel,
        getRefreshPlan,
        isLoading: network === null || isPriceLoading,
      }}
    >
      {children}
    </PriceContext.Provider>
  );
}

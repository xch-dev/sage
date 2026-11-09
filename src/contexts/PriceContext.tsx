import { useNetwork } from '@/hooks/useNetwork';
import { dexieApiUrl } from '@/lib/urls';
import {
  createContext,
  ReactNode,
  useCallback,
  useEffect,
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
  isLoading: boolean;
}

export const PriceContext = createContext<PriceContextType | undefined>(
  undefined,
);

export function PriceProvider({ children }: { children: ReactNode }) {
  const [xchUsdPrice, setXchUsdPrice] = useState<number>(0);
  const [catPrices, setCatPrices] = useState<Record<string, CatPriceData>>({});
  const { network, isTestnet } = useNetwork();
  const [isPriceLoading, setIsPriceLoading] = useState(false);
  const intervalRef = useRef<NodeJS.Timeout | null>(null);

  useEffect(() => {
    // Don't fetch prices until network is loaded
    if (network === null) {
      return;
    }

    if (intervalRef.current) {
      clearInterval(intervalRef.current);
      intervalRef.current = null;
    }

    const fetchCatPrices = async () => {
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

    const fetchPrices = async () => {
      setIsPriceLoading(true);
      try {
        await Promise.all([fetchCatPrices(), fetchChiaPrice()]);
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
  }, [network, isTestnet]);

  const getPriceInUsd = useCallback(
    (assetId: string | null) => {
      if (!assetId) return xchUsdPrice;

      const priceData = catPrices[assetId.toLowerCase()];
      const xchPrice = priceData?.lastPrice;

      if (xchPrice === null || xchPrice === undefined) {
        return 0;
      }

      return xchPrice * xchUsdPrice;
    },
    [xchUsdPrice, catPrices],
  );

  const getBalanceInUsd = useCallback(
    (assetId: string | null, balance: string) => {
      // Validate balance input
      const balanceNum = Number(balance);
      if (isNaN(balanceNum)) {
        return '0.00';
      }

      if (!assetId) return (balanceNum * xchUsdPrice).toFixed(2);

      const priceData = catPrices[assetId.toLowerCase()];
      const xchPrice = priceData?.lastPrice;

      if (xchPrice === null || xchPrice === undefined) {
        return '0.00';
      }

      return (balanceNum * xchPrice * xchUsdPrice).toFixed(2);
    },
    [xchUsdPrice, catPrices],
  );

  const getCatAskPriceInXch = useCallback(
    (assetId: string) => {
      const priceData = catPrices[assetId.toLowerCase()];
      return priceData?.askPrice ?? null;
    },
    [catPrices],
  );

  return (
    <PriceContext.Provider
      value={{
        getBalanceInUsd,
        getPriceInUsd,
        getCatAskPriceInXch,
        isLoading: network === null || isPriceLoading,
      }}
    >
      {children}
    </PriceContext.Provider>
  );
}

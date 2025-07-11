import { useWalletState } from '@/state';
import {
  createContext,
  ReactNode,
  useCallback,
  useEffect,
  useState,
} from 'react';
import { isXch } from '@/lib/utils';

// Add an interface for the price data structure
interface CatPriceData {
  lastPrice: number | null;
  askPrice: number | null;
}

interface CatListItem {
  asset_id: string;
  name: string;
  ticker: string;
  icon_url: string;
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
  getBalanceInUsd: (assetId: string, balance: string) => string;
  getPriceInUsd: (assetId: string) => number;
  getCatAskPriceInXch: (assetId: string) => number | null;
  getCatList: () => CatListItem[];
}

export const PriceContext = createContext<PriceContextType | undefined>(
  undefined,
);

export function PriceProvider({ children }: { children: ReactNode }) {
  const walletState = useWalletState();
  const [xchUsdPrice, setChiaPrice] = useState<number>(0);
  const [catPrices, setCatPrices] = useState<Record<string, CatPriceData>>({});
  const [catList, setCatList] = useState<CatListItem[]>([]);

  useEffect(() => {
    const fetchCatPrices = () =>
      fetch('https://api.dexie.space/v3/prices/tickers')
        .then((res) => res.json())
        .then((data: DexieResponse) => {
          const tickers = data.tickers.reduce(
            (acc: Record<string, CatPriceData>, ticker: DexieTicker) => {
              acc[ticker.base_currency] = {
                lastPrice: ticker.last_price ? Number(ticker.last_price) : null,
                askPrice: ticker.ask ? Number(ticker.ask) : null,
              };
              return acc;
            },
            {},
          );
          setCatPrices(tickers);

          // Extract unique CAT list
          const uniqueCats = Array.from(
            new Map(
              data.tickers.map((ticker: DexieTicker) => [
                ticker.base_currency,
                {
                  asset_id: ticker.base_currency,
                  name: ticker.base_name,
                  ticker: ticker.base_code,
                  icon_url: `https://icons.dexie.space/${ticker.base_currency}.webp`,
                },
              ]),
            ).values(),
          );
          setCatList(uniqueCats);
        })
        .catch(() => {
          setCatPrices({});
        });

    const fetchChiaPrice = () =>
      fetch(
        'https://api.coingecko.com/api/v3/simple/price?ids=chia&vs_currencies=usd',
      )
        .then((res) => res.json())
        .then((data) => {
          setChiaPrice(data.chia.usd || 0);
        })
        .catch(() => {
          setChiaPrice(0);
        });

    const fetchPrices = () => Promise.all([fetchCatPrices(), fetchChiaPrice()]);

    if (walletState.sync.unit.ticker === 'XCH') {
      fetchPrices();
      const interval = setInterval(fetchPrices, 60000);
      return () => clearInterval(interval);
    } else {
      setChiaPrice(0);
      setCatPrices({});
    }
  }, [walletState.sync.unit.ticker]);

  const getPriceInUsd = useCallback(
    (assetId: string) => {
      if (isXch(assetId)) {
        return xchUsdPrice;
      }
      const priceData = catPrices[assetId];
      const xchPrice = priceData?.lastPrice ?? 0;
      return xchPrice * xchUsdPrice;
    },
    [xchUsdPrice, catPrices],
  );

  const getBalanceInUsd = useCallback(
    (assetId: string, balance: string) => {
      if (isXch(assetId)) {
        return (Number(balance) * xchUsdPrice).toFixed(2);
      }
      const priceData = catPrices[assetId];
      const xchPrice = priceData?.lastPrice ?? 0;
      return (Number(balance) * xchPrice * xchUsdPrice).toFixed(2);
    },
    [xchUsdPrice, catPrices],
  );

  const getCatAskPriceInXch = useCallback(
    (assetId: string) => {
      const priceData = catPrices[assetId];
      return priceData?.askPrice ?? null;
    },
    [catPrices],
  );

  const getCatList = useCallback(() => {
    return catList;
  }, [catList]);

  return (
    <PriceContext.Provider
      value={{
        getBalanceInUsd,
        getPriceInUsd,
        getCatAskPriceInXch,
        getCatList,
      }}
    >
      {children}
    </PriceContext.Provider>
  );
}

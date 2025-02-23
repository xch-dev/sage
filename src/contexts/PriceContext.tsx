import { useWalletState } from '@/state';
import {
  createContext,
  ReactNode,
  useCallback,
  useEffect,
  useState,
} from 'react';

// Add an interface for the price data structure
interface CatPriceData {
  lastPrice: number | null;
  askPrice: number | null;
}

export interface PriceContextType {
  getBalanceInUsd: (assetId: string, balance: string) => string;
  getPriceInUsd: (assetId: string) => number;
  getCatAskPriceInXch: (assetId: string) => number | null;
}

export const PriceContext = createContext<PriceContextType | undefined>(
  undefined,
);

export function PriceProvider({ children }: { children: ReactNode }) {
  const walletState = useWalletState();
  const [xchUsdPrice, setChiaPrice] = useState<number>(0);
  const [catPrices, setCatPrices] = useState<Record<string, CatPriceData>>({});

  useEffect(() => {
    const fetchCatPrices = () =>
      fetch('https://api.dexie.space/v2/prices/tickers')
        .then((res) => res.json())
        .then((data) => {
          const tickers = data.tickers.reduce(
            (acc: Record<string, CatPriceData>, ticker: any) => {
              acc[ticker.base_id] = {
                lastPrice: ticker.last_price ? Number(ticker.last_price) : null,
                askPrice: ticker.ask ? Number(ticker.ask) : null,
              };
              return acc;
            },
            {},
          );
          setCatPrices(tickers);
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
      if (assetId === 'xch') {
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
      if (assetId === 'xch') {
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

  return (
    <PriceContext.Provider
      value={{ getBalanceInUsd, getPriceInUsd, getCatAskPriceInXch }}
    >
      {children}
    </PriceContext.Provider>
  );
}

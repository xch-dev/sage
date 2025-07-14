import { CatRecord, commands, NetworkKind } from '@/bindings';
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
  getCatList: () => CatRecord[];
}

export const PriceContext = createContext<PriceContextType | undefined>(
  undefined,
);

export function PriceProvider({ children }: { children: ReactNode }) {
  const walletState = useWalletState();
  const [xchUsdPrice, setChiaPrice] = useState<number>(0);
  const [catPrices, setCatPrices] = useState<Record<string, CatPriceData>>({});
  const [catList, setCatList] = useState<CatRecord[]>([]);
  const [network, setNetwork] = useState<NetworkKind | null>(null);
  const [isNetworkLoading, setIsNetworkLoading] = useState(true);

  // Fetch network on mount
  useEffect(() => {
    const fetchNetwork = async () => {
      try {
        setIsNetworkLoading(true);
        const data = await commands.getNetwork({});
        setNetwork(data.kind);
      } catch (error) {
        console.error('Failed to fetch network:', error);
        // Set a default network or handle error appropriately
        setNetwork('mainnet');
      } finally {
        setIsNetworkLoading(false);
      }
    };

    fetchNetwork();
  }, []);

  useEffect(() => {
    commands.getAllCats({}).then((data) => {
      setCatList(data.cats);
    });
  }, []);

  // Fetch prices when network is available and wallet is synced
  useEffect(() => {
    // Don't fetch prices until network is loaded
    if (isNetworkLoading || network === null) {
      return;
    }

    const fetchCatPrices = () =>
      fetch(
        `https://${network === 'testnet' ? 'api-testnet' : 'api'}.dexie.space/v3/prices/tickers`,
      )
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
        })
        .catch((error) => {
          console.error('Failed to fetch CAT prices:', error);
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
        .catch((error) => {
          console.error('Failed to fetch Chia price:', error);
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
  }, [walletState.sync.unit.ticker, network, isNetworkLoading]);

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

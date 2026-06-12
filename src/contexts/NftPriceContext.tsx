import { commands, events, NftCollectionRecord } from '@/bindings';
import { NO_COLLECTION_ID } from '@/hooks/useNftData';
import { useNetwork } from '@/hooks/useNetwork';
import { usePrices } from '@/hooks/usePrices';
import {
  CollectionValue,
  computeCollectionValues,
  computeTotalXch,
  FloorEntry,
  isFresh,
  sanitizeFloorCache,
} from '@/lib/nftValue';
import { mintGardenApiUrl } from '@/lib/urls';
import {
  createContext,
  ReactNode,
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
} from 'react';
import { useLocalStorage } from 'usehooks-ts';

export interface NftPriceContextType {
  nftTotalXch: number;
  nftTotalUsd: number;
  collectionValues: Record<string, CollectionValue>;
  isLoading: boolean;
  lastUpdated: number | null;
}

export const NftPriceContext = createContext<NftPriceContextType | undefined>(
  undefined,
);

const FLOOR_FETCH_CONCURRENCY = 4;
const FLOOR_FETCH_TIMEOUT_MS = 10000;
const REFRESH_INTERVAL_MS = 60000;
const SYNC_EVENT_DEBOUNCE_MS = 2000;

async function fetchFloor(
  collectionId: string,
  isTestnet: boolean,
): Promise<number | null> {
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), FLOOR_FETCH_TIMEOUT_MS);

  try {
    const response = await fetch(
      mintGardenApiUrl(`collections/${collectionId}`, isTestnet),
      { signal: controller.signal },
    );

    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }

    const data: { floor_price?: number | null } = await response.json();
    return typeof data.floor_price === 'number' &&
      Number.isFinite(data.floor_price) &&
      data.floor_price >= 0
      ? data.floor_price
      : null;
  } finally {
    clearTimeout(timer);
  }
}

async function fetchOwnedCollections(): Promise<NftCollectionRecord[]> {
  const pageSize = 100;
  const collections: NftCollectionRecord[] = [];
  let offset = 0;

  for (;;) {
    const response = await commands.getNftCollections({
      offset,
      limit: pageSize,
      include_hidden: false,
    });
    collections.push(...response.collections);

    if (
      collections.length >= response.total ||
      response.collections.length === 0
    ) {
      break;
    }

    offset += pageSize;
  }

  return collections;
}

export function NftPriceProvider({ children }: { children: ReactNode }) {
  const { network, isTestnet } = useNetwork();
  const { getPriceInUsd } = usePrices();

  const [collections, setCollections] = useState<NftCollectionRecord[]>([]);
  const [floors, setFloors] = useLocalStorage<Record<string, FloorEntry>>(
    `nft-floors-${isTestnet ? 'testnet' : 'mainnet'}`,
    {},
    {
      deserializer: (value) => sanitizeFloorCache(JSON.parse(value)),
    },
  );
  const [isLoading, setIsLoading] = useState(true);
  const [lastUpdated, setLastUpdated] = useState<number | null>(null);

  const refreshingRef = useRef(false);
  const floorsRef = useRef(floors);
  floorsRef.current = floors;

  const refresh = useCallback(async () => {
    if (refreshingRef.current) return;
    refreshingRef.current = true;

    try {
      const key = await commands.getKey({});
      if (!key?.key) {
        setCollections([]);
        return;
      }

      const owned = await fetchOwnedCollections();
      setCollections(owned);

      const now = Date.now();
      const stale = owned
        .map((collection) => collection.collection_id)
        .filter((id) => id !== NO_COLLECTION_ID)
        .filter((id) => !isFresh(floorsRef.current[id], now));

      if (stale.length > 0) {
        const queue = [...stale];
        const workers = Array.from(
          { length: Math.min(FLOOR_FETCH_CONCURRENCY, queue.length) },
          async () => {
            for (;;) {
              const id = queue.shift();
              if (id === undefined) return;

              try {
                const floorXch = await fetchFloor(id, isTestnet);
                setFloors((prev) => ({
                  ...prev,
                  [id]: { floorXch, fetchedAt: Date.now() },
                }));
              } catch (error) {
                // Keep any stale entry; it is retried on the next cycle
                console.error(`Failed to fetch floor for ${id}:`, error);
              }
            }
          },
        );
        await Promise.all(workers);
      }

      setLastUpdated(Date.now());
    } catch (error) {
      console.error('Failed to refresh NFT collections:', error);
    } finally {
      refreshingRef.current = false;
      setIsLoading(false);
    }
  }, [isTestnet, setFloors]);

  useEffect(() => {
    // Don't fetch until network is loaded, mirroring PriceContext
    if (network === null) {
      return;
    }

    setIsLoading(true);
    refresh();
    const interval = setInterval(refresh, REFRESH_INTERVAL_MS);

    return () => clearInterval(interval);
  }, [network, refresh]);

  useEffect(() => {
    let timer: NodeJS.Timeout | null = null;

    const unlistenPromise = events.syncEvent.listen((event) => {
      switch (event.payload.type) {
        case 'coin_state':
        case 'puzzle_batch_synced':
        case 'nft_data':
          if (timer) clearTimeout(timer);
          timer = setTimeout(() => refresh(), SYNC_EVENT_DEBOUNCE_MS);
          break;
      }
    });

    return () => {
      if (timer) clearTimeout(timer);
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, [refresh]);

  const collectionValues = useMemo(
    () => computeCollectionValues(collections, floors),
    [collections, floors],
  );

  const nftTotalXch = useMemo(
    () => computeTotalXch(collectionValues),
    [collectionValues],
  );

  const nftTotalUsd = nftTotalXch * getPriceInUsd(null);

  return (
    <NftPriceContext.Provider
      value={{
        nftTotalXch,
        nftTotalUsd,
        collectionValues,
        isLoading,
        lastUpdated,
      }}
    >
      {children}
    </NftPriceContext.Provider>
  );
}

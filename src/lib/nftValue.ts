import { NftCollectionRecord } from '@/bindings';
import { NO_COLLECTION_ID } from '@/hooks/useNftData';

export const FLOOR_TTL_MS = 15 * 60 * 1000;

export interface FloorEntry {
  floorXch: number | null;
  fetchedAt: number;
}

export interface CollectionValue {
  floorXch: number | null;
  count: number;
  valueXch: number;
}

export function isFresh(
  entry: FloorEntry | undefined,
  now: number,
): entry is FloorEntry {
  return entry !== undefined && now - entry.fetchedAt < FLOOR_TTL_MS;
}

export function computeCollectionValues(
  collections: NftCollectionRecord[],
  floors: Record<string, FloorEntry>,
): Record<string, CollectionValue> {
  const values: Record<string, CollectionValue> = {};

  for (const collection of collections) {
    // Uncategorized NFTs have no real collection, hence no floor
    if (collection.collection_id === NO_COLLECTION_ID) continue;

    const entry = floors[collection.collection_id];
    // Never-fetched collections are omitted from totals entirely
    if (!entry) continue;

    values[collection.collection_id] = {
      floorXch: entry.floorXch,
      count: collection.nft_count,
      valueXch: (entry.floorXch ?? 0) * collection.nft_count,
    };
  }

  return values;
}

export function computeTotalXch(
  values: Record<string, CollectionValue>,
): number {
  return Object.values(values).reduce(
    (total, value) => total + value.valueXch,
    0,
  );
}

function isFloorEntry(value: unknown): value is FloorEntry {
  if (value === null || typeof value !== 'object') return false;
  const { fetchedAt, floorXch } = value as Record<string, unknown>;
  return (
    typeof fetchedAt === 'number' &&
    Number.isFinite(fetchedAt) &&
    fetchedAt >= 0 &&
    (floorXch === null ||
      (typeof floorXch === 'number' &&
        Number.isFinite(floorXch) &&
        floorXch >= 0))
  );
}

export function sanitizeFloorCache(raw: unknown): Record<string, FloorEntry> {
  if (raw === null || typeof raw !== 'object' || Array.isArray(raw)) {
    return {};
  }

  const result: Record<string, FloorEntry> = {};
  for (const [key, value] of Object.entries(raw)) {
    if (isFloorEntry(value)) {
      result[key] = value;
    }
  }
  return result;
}

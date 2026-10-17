// The sources Sage reads prices and token labels from, in the user's order of
// priority. For each value (a CAT's price in XCH, the XCH/USD rate, a label for
// an unnamed CAT), the first enabled source that has it wins and later sources
// only fill gaps.
//
// The defaults reproduce Sage's long-standing behavior: Dexie for CAT prices,
// CoinGecko for XCH/USD, and nothing else. A user who never changes the
// setting makes exactly the requests Sage always made.

import {
  isIndexForNetwork,
  isValidTokenIndexUrl,
  TokenIndex,
  TokenIndexEntry,
} from './tokenIndex';

export type PriceSourceKind = 'dexie' | 'coingecko' | 'index';

export interface BuiltInPriceSource {
  id: string;
  name: string;
  kind: PriceSourceKind;
  homepage: string;
  enabledByDefault: boolean;
  /** For a built-in token index: its URL on each network. */
  indexUrls?: { mainnet: string; testnet11: string };
}

/** Built-in sources, in their default order of priority. */
export const BUILT_IN_PRICE_SOURCES: BuiltInPriceSource[] = [
  {
    id: 'dexie',
    name: 'Dexie',
    kind: 'dexie',
    homepage: 'https://dexie.space',
    enabledByDefault: true,
  },
  {
    id: 'coingecko',
    name: 'CoinGecko',
    kind: 'coingecko',
    homepage: 'https://www.coingecko.com',
    enabledByDefault: true,
  },
  // After Dexie and CoinGecko, so it only fills what they lack: prices from
  // the Forge's AMM pools and labels for Forge-traded CATs and LP tokens.
  {
    id: 'forge',
    name: 'The Forge',
    kind: 'index',
    homepage: 'https://forge.awizard.dev',
    enabledByDefault: true,
    indexUrls: {
      mainnet: 'https://forge.awizard.dev/token-index/v1/mainnet.json',
      testnet11: 'https://forge.awizard.dev/token-index/v1/testnet11.json',
    },
  },
];

/** One entry of the stored, ordered source list. */
export interface PriceSourceSetting {
  id: string;
  enabled: boolean;
  /** Set only for a custom token index. */
  url?: string;
}

/** An enabled source, resolved for the current network. */
export interface ActivePriceSource {
  id: string;
  kind: PriceSourceKind;
  /** The token index URL for an index source. */
  url?: string;
}

const CUSTOM_PREFIX = 'custom:';

export function customSourceId(url: string) {
  return `${CUSTOM_PREFIX}${url}`;
}

export function isCustomSource(source: PriceSourceSetting) {
  return source.id.startsWith(CUSTOM_PREFIX);
}

export function builtInSource(id: string) {
  return BUILT_IN_PRICE_SOURCES.find((source) => source.id === id) ?? null;
}

export function defaultPriceSources(): PriceSourceSetting[] {
  return BUILT_IN_PRICE_SOURCES.map((source) => ({
    id: source.id,
    enabled: source.enabledByDefault,
  }));
}

/**
 * Cleans a stored source list: drops unknown, duplicate and invalid entries,
 * and adds any built-in source the list does not have yet (a source added in
 * a later version of Sage) at its default place, with its default switch.
 */
export function normalizePriceSources(stored: unknown): PriceSourceSetting[] {
  if (!Array.isArray(stored)) return defaultPriceSources();

  const seen = new Set<string>();
  const sources: PriceSourceSetting[] = [];

  for (const entry of stored) {
    if (typeof entry !== 'object' || entry === null) continue;
    const { id, enabled, url } = entry as Record<string, unknown>;
    if (typeof id !== 'string' || seen.has(id)) continue;

    if (builtInSource(id)) {
      sources.push({ id, enabled: enabled !== false });
    } else if (
      typeof url === 'string' &&
      isValidTokenIndexUrl(url) &&
      id === customSourceId(url.trim())
    ) {
      sources.push({ id, enabled: enabled !== false, url: url.trim() });
    } else {
      continue;
    }

    seen.add(id);
  }

  BUILT_IN_PRICE_SOURCES.forEach((source, index) => {
    if (seen.has(source.id)) return;

    // Place it after every built-in that comes before it by default, so it
    // never outranks a source it was meant to follow.
    const at = BUILT_IN_PRICE_SOURCES.slice(0, index).reduce(
      (after, previous) =>
        Math.max(
          after,
          sources.findIndex((entry) => entry.id === previous.id) + 1,
        ),
      0,
    );

    sources.splice(at, 0, {
      id: source.id,
      enabled: source.enabledByDefault,
    });
    seen.add(source.id);
  });

  return sources;
}

/** Adds a custom token index at the end (lowest priority), enabled. */
export function addCustomSource(
  sources: PriceSourceSetting[],
  url: string,
): PriceSourceSetting[] | null {
  const trimmed = url.trim();
  if (!isValidTokenIndexUrl(trimmed)) return null;

  const id = customSourceId(trimmed);
  if (sources.some((source) => source.id === id)) return sources;

  return [...sources, { id, enabled: true, url: trimmed }];
}

/** Moves a source one place up (-1) or down (+1) in priority. */
export function moveSource(
  sources: PriceSourceSetting[],
  id: string,
  direction: -1 | 1,
): PriceSourceSetting[] {
  const from = sources.findIndex((source) => source.id === id);
  const to = from + direction;
  if (from === -1 || to < 0 || to >= sources.length) return sources;

  const next = [...sources];
  [next[from], next[to]] = [next[to], next[from]];
  return next;
}

/**
 * Reads the single token index URL from the earlier, unreleased version of
 * this setting (a JSON string under `tokenIndexUrl`) into the source list.
 */
export function migrateLegacyTokenIndexUrl(
  sources: PriceSourceSetting[],
  legacy: string | null,
): PriceSourceSetting[] {
  if (legacy === null) return sources;

  let url: unknown;
  try {
    url = JSON.parse(legacy);
  } catch {
    url = legacy;
  }

  if (typeof url !== 'string') return sources;
  return addCustomSource(sources, url) ?? sources;
}

/** The enabled sources, in priority order, resolved for the network. */
export function activePriceSources(
  sources: PriceSourceSetting[],
  isTestnet: boolean,
): ActivePriceSource[] {
  const active: ActivePriceSource[] = [];

  for (const source of sources) {
    if (!source.enabled) continue;

    if (source.url) {
      active.push({ id: source.id, kind: 'index', url: source.url });
      continue;
    }

    const builtIn = builtInSource(source.id);
    if (!builtIn) continue;

    if (builtIn.kind === 'index') {
      if (!builtIn.indexUrls) continue;
      active.push({
        id: builtIn.id,
        kind: 'index',
        url: isTestnet
          ? builtIn.indexUrls.testnet11
          : builtIn.indexUrls.mainnet,
      });
    } else {
      active.push({ id: builtIn.id, kind: builtIn.kind });
    }
  }

  return active;
}

/** What one source currently knows. */
export interface PriceLayer {
  /** XCH per whole token, by lowercase asset id. */
  catPricesXch?: Record<string, number | null | undefined>;
  xchUsd?: number | null;
  /** Labels for display, by lowercase asset id. */
  labels?: Record<string, TokenIndexEntry>;
}

/** A token index as a price layer, if it is for the wallet's network. */
export function indexLayer(
  index: TokenIndex | undefined,
  isTestnet: boolean,
): PriceLayer {
  if (!index || !isIndexForNetwork(index, isTestnet)) return {};

  const catPricesXch: Record<string, number | null> = {};
  for (const [assetId, entry] of Object.entries(index.tokens)) {
    catPricesXch[assetId] = entry.priceXch;
  }

  return { catPricesXch, xchUsd: index.xchUsd, labels: index.tokens };
}

/** The first layer's price for a CAT, in XCH. */
export function firstCatPriceXch(
  layers: PriceLayer[],
  assetId: string,
): number | null {
  const id = assetId.toLowerCase();
  for (const layer of layers) {
    const price = layer.catPricesXch?.[id];
    if (typeof price === 'number' && Number.isFinite(price)) return price;
  }
  return null;
}

/** The first layer's XCH/USD rate, or 0 if no layer has one. */
export function firstXchUsd(layers: PriceLayer[]): number {
  for (const layer of layers) {
    const rate = layer.xchUsd;
    if (typeof rate === 'number' && Number.isFinite(rate) && rate > 0) {
      return rate;
    }
  }
  return 0;
}

/** The first layer's suggested label (one with a name) for a CAT. */
export function firstLabel(
  layers: PriceLayer[],
  assetId: string,
): TokenIndexEntry | null {
  const id = assetId.toLowerCase();
  for (const layer of layers) {
    const entry = layer.labels?.[id];
    if (entry?.name) return entry;
  }
  return null;
}

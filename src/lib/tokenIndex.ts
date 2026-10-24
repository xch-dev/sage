// A token index is one JSON document that names and prices CATs, served by any
// provider at an https URL (Settings > Network > Price & Token Sources). Sage
// reads it on its price refresh; its prices count in the user's order of
// sources (see priceSources.ts), and its labels are only suggested, for display,
// for a CAT that has no name.
//
// Format (version 1):
//
//   {
//     "version": 1,
//     "provider": { "name": "...", "url": "https://..." },
//     "network": "mainnet" | "testnet11",
//     "generated_at": "<ISO 8601>",
//     "ttl_seconds": 300,
//     "xch_usd": 1.62 | null,
//     "tokens": [{
//       "asset_id": "<64 lowercase hex>",       required
//       "name": "...",                          1..64 chars
//       "ticker": "...",                        1..16 chars
//       "icon_url": "https://...",              https, <= 512 chars
//       "description": "...",                   <= 280 chars
//       "decimals": 3,                          0..18
//       "price_xch": 0.0000162,                 XCH per whole token, > 0
//       "price_usd": 0.0000262,                 USD per whole token, >= 0
//       "liquidity_xch": 91.8,                  >= 0
//       "source": "amm"                         informational
//     }]
//   }
//
// The text bounds match chia_updateCat's. A document with another version is
// refused; within a valid document, a bad entry or field is dropped on its own.

export interface TokenIndexEntry {
  name: string | null;
  ticker: string | null;
  iconUrl: string | null;
  priceXch: number | null;
}

export interface TokenIndex {
  network: string;
  xchUsd: number | null;
  tokens: Record<string, TokenIndexEntry>;
}

const MAX_TOKENS = 10_000;
const MAX_PRICE_XCH = 1e9;
const NATIVE_ASSET_ID = '0'.repeat(64);

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function text(value: unknown, max: number): string | null {
  if (typeof value !== 'string') return null;
  const trimmed = value.trim();
  return trimmed.length > 0 && trimmed.length <= max ? trimmed : null;
}

function httpsUrl(value: unknown): string | null {
  if (typeof value !== 'string' || value.length > 512) return null;
  if (!value.startsWith('https://')) return null;
  try {
    new URL(value);
    return value;
  } catch {
    return null;
  }
}

function positive(value: unknown, max: number): number | null {
  return typeof value === 'number' &&
    Number.isFinite(value) &&
    value > 0 &&
    value <= max
    ? value
    : null;
}

/** Checks a token index document, or returns null if it is not a valid v1 index. */
export function parseTokenIndex(data: unknown): TokenIndex | null {
  if (!isRecord(data) || data.version !== 1 || !Array.isArray(data.tokens)) {
    return null;
  }

  const tokens: Record<string, TokenIndexEntry> = {};

  for (const token of data.tokens.slice(0, MAX_TOKENS)) {
    if (!isRecord(token) || typeof token.asset_id !== 'string') continue;

    const assetId = token.asset_id;
    if (!/^[0-9a-f]{64}$/.test(assetId) || assetId === NATIVE_ASSET_ID) {
      continue;
    }

    tokens[assetId] = {
      name: text(token.name, 64),
      ticker: text(token.ticker, 16),
      iconUrl: httpsUrl(token.icon_url),
      priceXch: positive(token.price_xch, MAX_PRICE_XCH),
    };
  }

  return {
    network: typeof data.network === 'string' ? data.network : '',
    xchUsd: positive(data.xch_usd, MAX_PRICE_XCH),
    tokens,
  };
}

/** Whether an index document is for the network the wallet is on. */
export function isIndexForNetwork(index: TokenIndex, isTestnet: boolean) {
  return index.network === (isTestnet ? 'testnet11' : 'mainnet');
}

/**
 * A token as shown, with the index's name, ticker and icon each suggested only
 * where the token has none: a token named in the wallet but missing a ticker or
 * icon still gets them. A value from the user or Dexie is never replaced, and
 * nothing is written to the wallet.
 */
export function withSuggestedLabel<
  T extends {
    name: string | null;
    ticker: string | null;
    icon_url: string | null;
  },
>(token: T, entry: TokenIndexEntry | null): T {
  if (!entry) return token;

  const name = token.name || entry.name;
  const ticker = token.ticker || entry.ticker;
  const icon_url = token.icon_url || entry.iconUrl;

  if (
    name === token.name &&
    ticker === token.ticker &&
    icon_url === token.icon_url
  ) {
    return token;
  }

  return { ...token, name, ticker, icon_url };
}

/** Whether a URL can be used as a token index URL (https only). */
export function isValidTokenIndexUrl(url: string) {
  return httpsUrl(url.trim()) !== null;
}

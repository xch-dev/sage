import { describe, expect, it } from 'vitest';
import {
  isIndexForNetwork,
  isValidTokenIndexUrl,
  labelAsset,
  parseTokenIndex,
  withSuggestedLabel,
} from './tokenIndex';

const SPROUT =
  'ab558b1b841365a24d1ff2264c55982e55664a8b6e45bc107446b7e667bb463b';
const LP = 'f994df020df3b4616c445b5a2a955fbb4216beb1ce3ee3ac4aa75f8b2bc67db5';

function index(tokens: unknown[], extra: Record<string, unknown> = {}) {
  return {
    version: 1,
    provider: { name: 'Example', url: 'https://index.example' },
    network: 'mainnet',
    generated_at: '2026-10-17T00:00:00.000Z',
    ttl_seconds: 300,
    xch_usd: 1.62,
    tokens,
    ...extra,
  };
}

describe('parseTokenIndex', () => {
  it('reads a valid v1 index', () => {
    const parsed = parseTokenIndex(
      index([
        {
          asset_id: SPROUT,
          name: 'SPROUT',
          ticker: '🌱',
          icon_url: 'https://index.example/sprout.webp',
          decimals: 3,
          price_xch: 0.000016181075,
          price_usd: 0.0000262133,
          liquidity_xch: 91.86,
          source: 'amm',
        },
        {
          asset_id: LP,
          name: 'FLP - 🌱 XCH',
          ticker: '⚒️🌱XCH',
          icon_url: 'https://index.example/lp.png',
        },
      ]),
    );

    expect(parsed?.network).toBe('mainnet');
    expect(parsed?.xchUsd).toBe(1.62);
    expect(parsed?.tokens[SPROUT]).toEqual({
      name: 'SPROUT',
      ticker: '🌱',
      iconUrl: 'https://index.example/sprout.webp',
      priceXch: 0.000016181075,
    });
    expect(parsed?.tokens[LP]?.priceXch).toBeNull();
  });

  it('refuses another version or a missing token list', () => {
    expect(parseTokenIndex(index([], { version: 2 }))).toBeNull();
    expect(parseTokenIndex({ version: 1 })).toBeNull();
    expect(parseTokenIndex(null)).toBeNull();
    expect(parseTokenIndex([])).toBeNull();
  });

  it('drops bad entries and bad fields one at a time', () => {
    const parsed = parseTokenIndex(
      index(
        [
          { asset_id: SPROUT.toUpperCase(), name: 'Upper' },
          { asset_id: '0'.repeat(64), name: 'Chia' },
          { asset_id: 'abc', name: 'Short' },
          {
            asset_id: SPROUT,
            name: 'x'.repeat(65),
            ticker: 'y'.repeat(17),
            icon_url: 'http://index.example/a.png',
            price_xch: -1,
          },
          { asset_id: LP, name: ' Padded ', price_xch: Number.NaN },
        ],
        { xch_usd: 'a lot' },
      ),
    );

    expect(Object.keys(parsed?.tokens ?? {})).toEqual([SPROUT, LP]);
    expect(parsed?.tokens[SPROUT]).toEqual({
      name: null,
      ticker: null,
      iconUrl: null,
      priceXch: null,
    });
    expect(parsed?.tokens[LP]?.name).toBe('Padded');
    expect(parsed?.tokens[LP]?.priceXch).toBeNull();
    expect(parsed?.xchUsd).toBeNull();
  });

  it('matches the index to the wallet network', () => {
    const mainnet = parseTokenIndex(index([]));
    const testnet = parseTokenIndex(index([], { network: 'testnet11' }));

    if (!mainnet || !testnet) throw new Error('expected valid indexes');
    expect(isIndexForNetwork(mainnet, false)).toBe(true);
    expect(isIndexForNetwork(mainnet, true)).toBe(false);
    expect(isIndexForNetwork(testnet, true)).toBe(true);
  });
});

describe('withSuggestedLabel', () => {
  const entry = {
    name: 'SPROUT',
    ticker: '🌱',
    iconUrl: 'https://index.example/sprout.webp',
    priceXch: null,
  };

  it('suggests a label for a token with no name', () => {
    expect(
      withSuggestedLabel({ name: null, ticker: null, icon_url: null }, entry),
    ).toEqual({
      name: 'SPROUT',
      ticker: '🌱',
      icon_url: 'https://index.example/sprout.webp',
    });
  });

  it('fills a missing ticker and icon on a token the wallet already named', () => {
    expect(
      withSuggestedLabel(
        { name: 'My Sprout', ticker: null, icon_url: null },
        entry,
      ),
    ).toEqual({
      name: 'My Sprout',
      ticker: '🌱',
      icon_url: 'https://index.example/sprout.webp',
    });
  });

  it('never replaces an existing label', () => {
    const labeled = {
      name: 'My Sprout',
      ticker: 'MINE',
      icon_url: 'https://mine.example/a.png',
    };
    expect(withSuggestedLabel(labeled, entry)).toBe(labeled);

    expect(
      withSuggestedLabel(
        { name: null, ticker: 'MINE', icon_url: 'https://mine.example/a.png' },
        entry,
      ),
    ).toEqual({
      name: 'SPROUT',
      ticker: 'MINE',
      icon_url: 'https://mine.example/a.png',
    });
  });

  it('suggests nothing without an index entry, and only what the entry has', () => {
    const token = { name: null, ticker: null, icon_url: null };
    expect(withSuggestedLabel(token, null)).toBe(token);
    expect(withSuggestedLabel(token, { ...entry, name: null })).toEqual({
      name: null,
      ticker: '🌱',
      icon_url: 'https://index.example/sprout.webp',
    });
  });
});

describe('isValidTokenIndexUrl', () => {
  it('accepts https only', () => {
    expect(isValidTokenIndexUrl('https://index.example/v1/mainnet.json')).toBe(
      true,
    );
    expect(isValidTokenIndexUrl('http://index.example/v1/mainnet.json')).toBe(
      false,
    );
    expect(isValidTokenIndexUrl('not a url')).toBe(false);
    expect(isValidTokenIndexUrl('')).toBe(false);
  });
});

describe('labelAsset', () => {
  const entry = {
    name: 'Forge LP',
    ticker: 'FLP',
    iconUrl: 'https://i.example/lp.png',
    priceXch: null,
  };
  const suggest = (assetId: string) => (assetId === LP ? entry : null);
  const cat = (name: string | null, kind = 'token') => ({
    asset_id: LP,
    name,
    ticker: null,
    icon_url: null,
    kind,
  });

  it('shows an unnamed CAT with the suggestion, in a transaction row as on the token list', () => {
    expect(labelAsset(cat(null), suggest)).toMatchObject({
      name: 'Forge LP',
      ticker: 'FLP',
      icon_url: entry.iconUrl,
    });
  });

  it('keeps a name the wallet has, and only fills what is missing', () => {
    expect(labelAsset(cat('Mine'), suggest)).toMatchObject({
      name: 'Mine',
      ticker: 'FLP',
    });
  });

  it('leaves XCH, NFTs and unknown asset ids alone', () => {
    expect(
      labelAsset({ ...cat(null), asset_id: null }, suggest).name,
    ).toBeNull();
    expect(labelAsset(cat(null, 'nft'), suggest).name).toBeNull();
    expect(
      labelAsset({ ...cat(null), asset_id: SPROUT }, suggest).name,
    ).toBeNull();
  });

  it('works for records with no kind, such as token records', () => {
    const record = { asset_id: LP, name: null, ticker: null, icon_url: null };
    expect(labelAsset(record, suggest).name).toBe('Forge LP');
  });
});

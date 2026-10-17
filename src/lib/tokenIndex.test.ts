import { describe, expect, it } from 'vitest';
import {
  isIndexForNetwork,
  isValidTokenIndexUrl,
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

  it('never replaces an existing label', () => {
    const named = { name: 'My Sprout', ticker: null, icon_url: null };
    expect(withSuggestedLabel(named, entry)).toBe(named);

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

  it('suggests nothing without an index name', () => {
    const token = { name: null, ticker: null, icon_url: null };
    expect(withSuggestedLabel(token, null)).toBe(token);
    expect(withSuggestedLabel(token, { ...entry, name: null })).toBe(token);
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

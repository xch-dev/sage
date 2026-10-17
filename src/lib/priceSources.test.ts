import { describe, expect, it } from 'vitest';
import {
  activePriceSources,
  addCustomSource,
  BUILT_IN_PRICE_SOURCES,
  customSourceId,
  defaultPriceSources,
  firstCatPriceXch,
  firstLabel,
  firstXchUsd,
  indexLayer,
  migrateLegacyTokenIndexUrl,
  moveSource,
  normalizePriceSources,
  PriceLayer,
} from './priceSources';
import { parseTokenIndex } from './tokenIndex';

const SPROUT =
  'ab558b1b841365a24d1ff2264c55982e55664a8b6e45bc107446b7e667bb463b';
const OTHER =
  'f994df020df3b4616c445b5a2a955fbb4216beb1ce3ee3ac4aa75f8b2bc67db5';
const URL_A = 'https://a.example/v1/mainnet.json';
const URL_B = 'https://b.example/v1/mainnet.json';

const label = (name: string | null) => ({
  name,
  ticker: null,
  iconUrl: null,
  priceXch: null,
});

describe('defaults', () => {
  it('are Dexie, CoinGecko, then The Forge, all on', () => {
    expect(normalizePriceSources(null)).toEqual(defaultPriceSources());
    expect(activePriceSources(defaultPriceSources(), false)).toEqual([
      { id: 'dexie', kind: 'dexie' },
      { id: 'coingecko', kind: 'coingecko' },
      {
        id: 'forge',
        kind: 'index',
        url: 'https://forge.awizard.dev/token-index/v1/mainnet.json',
      },
    ]);
    expect(activePriceSources(defaultPriceSources(), true).at(-1)?.url).toBe(
      'https://forge.awizard.dev/token-index/v1/testnet11.json',
    );
  });

  it('keep Dexie and CoinGecko ahead of The Forge', () => {
    const dexie: PriceLayer = { catPricesXch: { [SPROUT]: 0.5 } };
    const coingecko: PriceLayer = { xchUsd: 10 };
    const forge: PriceLayer = {
      catPricesXch: { [SPROUT]: 0.25, [OTHER]: 0.1 },
      xchUsd: 12,
    };
    const layers = [dexie, coingecko, forge];

    expect(firstCatPriceXch(layers, SPROUT)).toBe(0.5);
    expect(firstCatPriceXch(layers, OTHER)).toBe(0.1);
    expect(firstXchUsd(layers)).toBe(10);
  });

  it('turn The Forge on for a list saved before it existed', () => {
    const saved = [
      { id: 'coingecko', enabled: true },
      { id: 'dexie', enabled: false },
      { id: customSourceId(URL_A), enabled: true, url: URL_A },
    ];

    expect(normalizePriceSources(saved)).toEqual([
      ...saved.slice(0, 2),
      { id: 'forge', enabled: true },
      saved[2],
    ]);
  });
});

describe('normalizePriceSources', () => {
  it('drops unknown, duplicate and invalid entries', () => {
    const sources = normalizePriceSources([
      { id: 'coingecko', enabled: false },
      { id: 'dexie', enabled: true },
      { id: 'dexie', enabled: false },
      { id: 'mystery', enabled: true },
      { id: customSourceId(URL_A), enabled: true, url: URL_A },
      { id: customSourceId(URL_A), enabled: false, url: URL_A },
      {
        id: customSourceId('http://a.example'),
        enabled: true,
        url: 'http://a.example',
      },
      { id: 'custom:wrong', enabled: true, url: URL_B },
      'junk',
      null,
    ]);

    const kept = ['coingecko', 'dexie', customSourceId(URL_A)];
    expect(sources.filter((source) => kept.includes(source.id))).toEqual([
      { id: 'coingecko', enabled: false },
      { id: 'dexie', enabled: true },
      { id: customSourceId(URL_A), enabled: true, url: URL_A },
    ]);
    expect(sources).toHaveLength(BUILT_IN_PRICE_SOURCES.length + 1);
  });

  it('adds a missing built-in after the built-ins before it', () => {
    const sources = normalizePriceSources([
      { id: customSourceId(URL_A), enabled: true, url: URL_A },
      { id: 'dexie', enabled: false },
      { id: customSourceId(URL_B), enabled: true, url: URL_B },
    ]);

    expect(sources.map((source) => source.id).slice(0, 3)).toEqual([
      customSourceId(URL_A),
      'dexie',
      'coingecko',
    ]);
    expect(sources.at(-1)?.id).toBe(customSourceId(URL_B));
    expect(sources[1].enabled).toBe(false);
    expect(sources[2].enabled).toBe(true);
  });
});

describe('editing the list', () => {
  it('adds https custom sources once, last', () => {
    const added = addCustomSource(defaultPriceSources(), ` ${URL_A} `);
    expect(added?.at(-1)).toEqual({
      id: customSourceId(URL_A),
      enabled: true,
      url: URL_A,
    });
    expect(addCustomSource(added ?? [], URL_A)).toBe(added);
    expect(addCustomSource(defaultPriceSources(), 'http://a.example')).toBe(
      null,
    );
  });

  it('moves a source within bounds', () => {
    const sources = defaultPriceSources();
    const moved = moveSource(sources, 'coingecko', -1);
    expect(moved.map((source) => source.id).slice(0, 2)).toEqual([
      'coingecko',
      'dexie',
    ]);
    expect(moveSource(sources, 'dexie', -1)).toBe(sources);
    expect(moveSource(sources, 'missing', 1)).toBe(sources);
  });
});

describe('migrateLegacyTokenIndexUrl', () => {
  it('reads the old single URL as a custom source', () => {
    const migrated = migrateLegacyTokenIndexUrl(
      defaultPriceSources(),
      JSON.stringify(URL_A),
    );
    expect(migrated.at(-1)?.url).toBe(URL_A);
  });

  it('ignores an empty, missing or invalid old URL', () => {
    const sources = defaultPriceSources();
    expect(migrateLegacyTokenIndexUrl(sources, null)).toBe(sources);
    expect(migrateLegacyTokenIndexUrl(sources, '""')).toBe(sources);
    expect(migrateLegacyTokenIndexUrl(sources, '"http://a.example"')).toBe(
      sources,
    );
    expect(migrateLegacyTokenIndexUrl(sources, '{')).toBe(sources);
  });
});

describe('activePriceSources', () => {
  it('keeps enabled sources in order', () => {
    const sources = [
      { id: customSourceId(URL_A), enabled: true, url: URL_A },
      { id: 'dexie', enabled: false },
      { id: 'coingecko', enabled: true },
    ];

    expect(activePriceSources(sources, false)).toEqual([
      { id: customSourceId(URL_A), kind: 'index', url: URL_A },
      { id: 'coingecko', kind: 'coingecko' },
    ]);
  });
});

describe('priority and fallback', () => {
  const dexie: PriceLayer = { catPricesXch: { [SPROUT]: 0.5, [OTHER]: null } };
  const coingecko: PriceLayer = { xchUsd: 10 };
  const index: PriceLayer = {
    catPricesXch: { [SPROUT]: 0.25, [OTHER]: 0.1 },
    xchUsd: 12,
    labels: { [SPROUT]: label('Sprout'), [OTHER]: label(null) },
  };

  it('takes the first source with a value', () => {
    expect(firstCatPriceXch([dexie, coingecko, index], SPROUT)).toBe(0.5);
    expect(firstCatPriceXch([index, dexie], SPROUT)).toBe(0.25);
    expect(firstXchUsd([dexie, coingecko, index])).toBe(10);
    expect(firstXchUsd([index, coingecko])).toBe(12);
  });

  it('lets later sources fill gaps only', () => {
    expect(firstCatPriceXch([dexie, coingecko, index], OTHER)).toBe(0.1);
    expect(firstCatPriceXch([dexie], OTHER)).toBeNull();
    expect(firstXchUsd([{ xchUsd: 0 }, index])).toBe(12);
    expect(firstXchUsd([dexie])).toBe(0);
  });

  it('is case-insensitive on asset ids', () => {
    expect(firstCatPriceXch([dexie], SPROUT.toUpperCase())).toBe(0.5);
  });

  it('suggests the first label that has a name', () => {
    const second: PriceLayer = { labels: { [OTHER]: label('Other') } };
    expect(firstLabel([dexie, index, second], OTHER)?.name).toBe('Other');
    expect(firstLabel([dexie, index, second], SPROUT)?.name).toBe('Sprout');
    expect(firstLabel([dexie], SPROUT)).toBeNull();
  });

  it('ignores an index for the other network', () => {
    const parsed = parseTokenIndex({
      version: 1,
      network: 'mainnet',
      xch_usd: 5,
      tokens: [{ asset_id: SPROUT, name: 'Sprout', price_xch: 0.3 }],
    });

    expect(
      firstCatPriceXch([indexLayer(parsed ?? undefined, false)], SPROUT),
    ).toBe(0.3);
    expect(indexLayer(parsed ?? undefined, true)).toEqual({});
    expect(indexLayer(undefined, false)).toEqual({});
  });
});

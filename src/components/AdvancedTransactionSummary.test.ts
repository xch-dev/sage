// @vitest-environment jsdom

import { TransactionSummary, Unit } from '@/bindings';
import { i18n } from '@lingui/core';
import { afterEach, beforeAll, describe, expect, it, vi } from 'vitest';
import {
  calculateTransaction,
  groupSentAssets,
} from './AdvancedTransactionSummary';

vi.mock('@/state', () => ({ useWalletState: vi.fn() }));

const xch = { ticker: 'XCH', precision: 12 } as Unit;
const asset = { asset_id: null, kind: 'token' };

function output(coin_id: string, amount: number) {
  return { coin_id, amount, address: 'xch1', receiving: true, burning: false };
}

function input(coin_id: string, amount: number, outputs: unknown[]) {
  return { coin_id, amount, address: 'xch1', asset, outputs };
}

beforeAll(() => {
  i18n.loadAndActivate({ locale: 'en-US', messages: {} });
});

describe('calculateTransaction', () => {
  it('does not report coins created and spent in the transaction as spent', () => {
    const summary = {
      fee: 0,
      handles: [],
      inputs: [
        input('base', 1, [output('settlement', 0), output('change', 1)]),
        input('settlement', 0, [output('payment', 5000)]),
        input('payment', 5000, []),
      ],
    } as unknown as TransactionSummary;

    const { spent, created } = calculateTransaction(xch, summary);

    expect(spent.map((coin) => coin.coinId)).toEqual(['base']);
    expect(created).toHaveLength(1);
  });
});

function send(address: string, amount: number | string) {
  return {
    coin_id: `${address}-${amount}`,
    amount,
    address,
    receiving: false,
    burning: false,
  };
}

function spend(coinAsset: unknown, outputs: unknown[]) {
  return {
    coin_id: 'parent',
    amount: 0,
    address: 'me',
    asset: coinAsset,
    outputs,
  };
}

function sentGroups(inputs: unknown[]) {
  const summary = {
    fee: 0,
    handles: [],
    inputs,
  } as unknown as TransactionSummary;
  const { created } = calculateTransaction(xch, summary);
  return groupSentAssets(created).map(({ badge, total, recipients }) => ({
    badge,
    total,
    recipients,
  }));
}

function cat(asset_id: string, name = 'Token', ticker = 'TKN') {
  return { asset_id, name, ticker, kind: 'token' };
}

describe('groupSentAssets', () => {
  afterEach(() => {
    vi.restoreAllMocks();
  });

  function useLocale(locale: string) {
    vi.spyOn(navigator, 'language', 'get').mockReturnValue(locale);
  }

  it.each([
    ['en-US', '2,000 XCH', '3 TKN'],
    ['de-DE', '2.000 XCH', '3 TKN'],
  ])('sums every output of an asset in %s', (locale, xchTotal, catTotal) => {
    useLocale(locale);

    const groups = sentGroups([
      spend(asset, [
        send('alice', '1000000000000000'),
        send('bob', '999000000000000'),
        // A second output to the same address still counts.
        send('alice', '1000000000000'),
      ]),
      spend(cat('a1'), [send('alice', 1500), send('bob', 1500)]),
    ]);

    expect(groups).toEqual([
      { badge: 'Chia', total: xchTotal, recipients: ['alice', 'bob'] },
      { badge: 'CAT Token', total: catTotal, recipients: ['alice', 'bob'] },
    ]);
  });

  it('formats fractional totals with the locale decimal separator', () => {
    useLocale('de-DE');

    const groups = sentGroups([
      spend(cat('a1'), [
        send('alice', 1500),
        send('alice', 1500),
        send('bob', 250),
      ]),
    ]);

    expect(groups).toEqual([
      { badge: 'CAT Token', total: '3,25 TKN', recipients: ['alice', 'bob'] },
    ]);
  });

  it('sums XCH amounts without floating point error', () => {
    useLocale('en-US');

    const groups = sentGroups([
      spend(asset, [
        send('alice', '100000000000'),
        send('bob', '200000000000'),
      ]),
    ]);

    expect(groups[0].total).toBe('0.3 XCH');
  });

  it('keeps CATs with the same name and ticker separate', () => {
    useLocale('en-US');

    const groups = sentGroups([
      spend(cat('a1'), [send('alice', 1000)]),
      spend(cat('a2'), [send('alice', 2000)]),
    ]);

    expect(groups.map((group) => group.total)).toEqual(['1 TKN', '2 TKN']);
  });

  it('combines outputs of one CAT spread across several coins', () => {
    useLocale('en-US');

    const groups = sentGroups([
      spend(cat('a1'), [send('alice', 1000)]),
      { ...spend(cat('a1'), [send('bob', 2000)]), coin_id: 'other' },
    ]);

    expect(groups).toEqual([
      { badge: 'CAT Token', total: '3 TKN', recipients: ['alice', 'bob'] },
    ]);
  });

  it('lists each NFT on its own, labelled by name', () => {
    useLocale('en-US');

    const groups = sentGroups([
      spend({ asset_id: 'n1', name: 'Cool NFT #1', kind: 'nft' }, [
        send('alice', 1),
      ]),
      spend({ asset_id: 'n2', name: 'Cool NFT #1', kind: 'nft' }, [
        send('bob', 1),
      ]),
    ]);

    expect(groups).toEqual([
      { badge: 'NFT', total: 'Cool NFT #1', recipients: ['alice'] },
      { badge: 'NFT', total: 'Cool NFT #1', recipients: ['bob'] },
    ]);
  });
});

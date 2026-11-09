import type { AssetKind, OfferAsset, OfferRecord } from '@/bindings';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const getOffer = vi.fn();

vi.mock('@/bindings', () => ({
  commands: { getOffer: (...args: unknown[]) => getOffer(...args) },
}));

const { canCopyOffer, fetchOfferRecords, offerStateFromRecord } =
  await import('./offers');

beforeEach(() => getOffer.mockReset());

describe('fetchOfferRecords', () => {
  it('returns records in selection order', async () => {
    getOffer.mockImplementation((arg: { offer_id?: string } = {}) => {
      const { offer_id } = arg;
      return Promise.resolve({ offer: { offer_id, status: 'active' } });
    });
    const records = await fetchOfferRecords(['a', 'b']);
    expect(records.map((r) => r.offer_id)).toEqual(['a', 'b']);
  });

  it('skips offers that no longer exist', async () => {
    getOffer.mockImplementation((arg: { offer_id?: string } = {}) => {
      const { offer_id } = arg;
      return offer_id === 'gone'
        ? Promise.reject({ kind: 'not_found', reason: 'missing' })
        : Promise.resolve({ offer: { offer_id, status: 'active' } });
    });
    const records = await fetchOfferRecords(['a', 'gone', 'b']);
    expect(records.map((r) => r.offer_id)).toEqual(['a', 'b']);
  });

  it('does not report a not_found rejection to onError', async () => {
    getOffer.mockImplementation((arg: { offer_id?: string } = {}) => {
      const { offer_id } = arg;
      return offer_id === 'gone'
        ? Promise.reject({ kind: 'not_found', reason: 'missing' })
        : Promise.resolve({ offer: { offer_id, status: 'active' } });
    });
    const onError = vi.fn();
    const records = await fetchOfferRecords(['a', 'gone'], onError);
    expect(records.map((r) => r.offer_id)).toEqual(['a']);
    expect(onError).not.toHaveBeenCalled();
  });

  it('reports a non-not_found rejection to onError and still skips it', async () => {
    getOffer.mockImplementation((arg: { offer_id?: string } = {}) => {
      const { offer_id } = arg;
      return offer_id === 'broken'
        ? Promise.reject({ kind: 'internal', reason: 'boom' })
        : Promise.resolve({ offer: { offer_id, status: 'active' } });
    });
    const onError = vi.fn();
    const records = await fetchOfferRecords(['a', 'broken'], onError);
    expect(records.map((r) => r.offer_id)).toEqual(['a']);
    expect(onError).toHaveBeenCalledTimes(1);
    expect(onError).toHaveBeenCalledWith({ kind: 'internal', reason: 'boom' });
  });

  it('does not throw when onError is omitted for a non-not_found rejection', async () => {
    getOffer.mockImplementation((arg: { offer_id?: string } = {}) => {
      const { offer_id } = arg;
      return offer_id === 'broken'
        ? Promise.reject({ kind: 'internal', reason: 'boom' })
        : Promise.resolve({ offer: { offer_id, status: 'active' } });
    });
    const records = await fetchOfferRecords(['a', 'broken']);
    expect(records.map((r) => r.offer_id)).toEqual(['a']);
  });
});

function offerAsset(
  kind: AssetKind,
  assetId: string | null,
  amount: number,
): OfferAsset {
  return {
    asset: {
      asset_id: assetId,
      name: null,
      ticker: null,
      precision: assetId ? 3 : 12,
      icon_url: null,
      description: null,
      is_sensitive_content: false,
      is_visible: true,
      revocation_address: null,
      kind,
    },
    amount,
    royalty: 0,
    nft_royalty: null,
    option_assets: null,
    nft_metadata_updater_kind: null,
  };
}

function offerRecord(
  summary: Partial<OfferRecord['summary']>,
  creationTimestamp = 1_000_000,
): OfferRecord {
  return {
    offer_id: 'offer',
    offer: 'offer1...',
    status: 'active',
    creation_timestamp: creationTimestamp,
    summary: {
      fee: 0,
      maker: [],
      taker: [],
      expiration_height: null,
      expiration_timestamp: null,
      ...summary,
    },
  };
}

describe('offerStateFromRecord', () => {
  it('converts amounts and the fee from mojos', () => {
    const state = offerStateFromRecord(
      offerRecord({
        fee: 250_000_000,
        maker: [offerAsset('token', 'cat', 12_345)],
        taker: [offerAsset('token', null, 1_500_000_000_000)],
      }),
    );
    expect(state.offered.tokens).toEqual([
      { asset_id: 'cat', amount: '12.345' },
    ]);
    expect(state.requested.tokens).toEqual([{ asset_id: null, amount: '1.5' }]);
    expect(state.fee).toBe('0.00025');
    expect(state.expiration).toBeNull();
  });

  it("keeps the original offer's duration rather than its timestamp", () => {
    const created = 1_000_000;
    const state = offerStateFromRecord(
      offerRecord(
        { expiration_timestamp: created + 26 * 60 * 60 + 5 * 60 },
        created,
      ),
    );
    expect(state.expiration).toEqual({ days: '1', hours: '2', minutes: '5' });
  });

  it('drops a block height expiration', () => {
    const state = offerStateFromRecord(
      offerRecord({ expiration_height: 5_000_000 }),
    );
    expect(state.expiration).toBeNull();
  });

  it('separates NFTs and options from tokens', () => {
    const state = offerStateFromRecord(
      offerRecord({
        maker: [offerAsset('nft', 'nft1abc', 1)],
        taker: [offerAsset('option', 'option1abc', 1)],
      }),
    );
    expect(state.offered).toEqual({
      tokens: [],
      nfts: ['nft1abc'],
      options: [],
    });
    expect(state.requested).toEqual({
      tokens: [],
      nfts: [],
      options: ['option1abc'],
    });
  });
});

describe('canCopyOffer', () => {
  const tokens = {
    tokens: [{ asset_id: null, amount: '1' }],
    nfts: [],
    options: [],
  };

  it('allows offers made only of tokens', () => {
    expect(
      canCopyOffer({
        offered: tokens,
        requested: tokens,
        fee: '0',
        expiration: null,
      }),
    ).toBe(true);
  });

  it('rejects an NFT or option on either side', () => {
    const withNft = { ...tokens, nfts: ['nft1abc'] };
    const withOption = { ...tokens, options: ['option1abc'] };
    for (const [offered, requested] of [
      [withNft, tokens],
      [tokens, withNft],
      [withOption, tokens],
      [tokens, withOption],
    ]) {
      expect(
        canCopyOffer({ offered, requested, fee: '0', expiration: null }),
      ).toBe(false);
    }
  });

  it('ignores empty NFT and option slots', () => {
    const blank = { ...tokens, nfts: [''], options: [''] };
    expect(
      canCopyOffer({
        offered: blank,
        requested: blank,
        fee: '0',
        expiration: null,
      }),
    ).toBe(true);
  });
});

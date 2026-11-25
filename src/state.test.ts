import { afterEach, describe, expect, it, vi } from 'vitest';

vi.mock('./bindings', () => ({
  commands: {},
  events: { syncEvent: { listen: vi.fn(() => Promise.resolve(vi.fn())) } },
}));

import { OfferState, splitOfferNfts, useOfferState } from './state';

const offer: OfferState = {
  fee: '0',
  offered: { tokens: [], nfts: ['nft1a', 'nft1b'], options: [] },
  requested: { tokens: [], nfts: [], options: [] },
  expiration: null,
  splitNftOffers: false,
  preSplit: false,
  preSplitFee: '0',
};

afterEach(() => useOfferState.setState(null));

describe('splitOfferNfts', () => {
  it('turns on one offer per NFT for an offer being made', () => {
    useOfferState.setState(offer);
    splitOfferNfts();
    expect(useOfferState.getState()).toEqual({
      ...offer,
      splitNftOffers: true,
    });
  });

  it('leaves a cleared offer cleared', () => {
    splitOfferNfts();
    expect(useOfferState.getState()).toBeNull();
  });
});

// @vitest-environment jsdom

import { i18n } from '@lingui/core';
import { act, cleanup, renderHook } from '@testing-library/react';
import { afterEach, beforeAll, describe, expect, it, vi } from 'vitest';
import type { OfferState } from '@/state';

const bindings = vi.hoisted(() => ({
  makeOffer: vi.fn(),
  getFeeRate: vi.fn(),
}));

vi.mock('@/bindings', () => ({ commands: bindings }));
vi.mock('@/hooks/useBiometric', () => ({
  useBiometric: () => ({ promptIfEnabled: () => Promise.resolve(true) }),
}));
vi.mock('@/state', () => ({
  useWalletState: () => ({ sync: { unit: { precision: 12 } } }),
}));

import { useOfferProcessor } from './useOfferProcessor';

beforeAll(() => {
  i18n.loadAndActivate({ locale: 'en', messages: {} });
});

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

const offerState: OfferState = {
  fee: '0.001',
  expiration: null,
  offered: {
    tokens: [{ asset_id: null, amount: '1' }],
    nfts: [],
    options: [],
  },
  requested: {
    tokens: [{ asset_id: null, amount: '2' }],
    nfts: [],
    options: [],
  },
  splitNftOffers: false,
  preSplit: true,
  preSplitFee: '',
} as unknown as OfferState;

async function makeOfferRequest(preSplitFee: string | null) {
  bindings.getFeeRate.mockResolvedValue({
    fee_per_cost: 5,
    minimum_fee_per_cost: 5,
    tiers: [
      { name: 'fast', target_time: 60, fee_per_cost: 12 },
      { name: 'standard', target_time: 300, fee_per_cost: 8 },
      { name: 'economy', target_time: 3600, fee_per_cost: 5 },
    ],
  });
  bindings.makeOffer.mockResolvedValue({
    offer: 'offer1',
    offer_id: 'id',
    pre_split_transaction_id: null,
  });

  const { result } = renderHook(() =>
    useOfferProcessor({ offerState, splitNftOffers: false, preSplitFee }),
  );

  await act(() => result.current.processOffer());

  expect(bindings.makeOffer).toHaveBeenCalledTimes(1);
  return bindings.makeOffer.mock.calls[0][0];
}

describe('useOfferProcessor', () => {
  it("prices an automatic split fee at its tier's rate", async () => {
    const request = await makeOfferRequest('auto:standard');
    expect(request.pre_split_fee).toBe('0');
    expect(request.pre_split_fee_per_cost).toBe(8);
    // The offer's own fee is always an amount.
    expect(request.fee).toBe('1000000000');
  });

  it('prices an automatic split fee at a chosen rate', async () => {
    const request = await makeOfferRequest('auto:7');
    expect(request.pre_split_fee).toBe('0');
    expect(request.pre_split_fee_per_cost).toBe(7);
  });

  it('pays an entered split fee as is', async () => {
    const request = await makeOfferRequest('0.0001');
    expect(request.pre_split_fee).toBe('100000000');
    expect(request.pre_split_fee_per_cost).toBeNull();
  });

  it("doesn't split without a split fee", async () => {
    const request = await makeOfferRequest(null);
    expect(request.pre_split_fee).toBeNull();
    expect(request.pre_split_fee_per_cost).toBeNull();
  });
});

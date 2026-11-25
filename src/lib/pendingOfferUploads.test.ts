// @vitest-environment jsdom

import { i18n } from '@lingui/core';
import { beforeEach, describe, expect, it, vi } from 'vitest';

i18n.loadAndActivate({ locale: 'en', messages: {} });

const getOffer = vi.fn();
const getWallet = vi.fn();
const uploadToDexie = vi.fn();
const uploadToMintGarden = vi.fn();

vi.mock('@/bindings', () => ({
  commands: {
    getOffer: (...args: unknown[]) => getOffer(...args),
    getWallet: (...args: unknown[]) => getWallet(...args),
  },
}));

vi.mock('@/lib/marketplaces', () => ({
  marketplaces: [
    { id: 'dexie', name: 'Dexie', uploadToMarketplace: uploadToDexie },
    {
      id: 'mintgarden',
      name: 'MintGarden',
      uploadToMarketplace: uploadToMintGarden,
    },
  ],
}));

vi.mock('react-toastify', () => ({
  toast: { success: vi.fn(), error: vi.fn() },
}));

const {
  MAX_UPLOAD_FAILURES,
  processPendingOfferUploads,
  queuePendingOfferUpload,
} = await import('./pendingOfferUploads');

function queue(offerId: string, fingerprint = 1, marketplaceIds = ['dexie']) {
  queuePendingOfferUpload({
    fingerprint,
    offerId,
    offer: `offer-${offerId}`,
    isTestnet: true,
    marketplaceIds,
  });
}

function queued() {
  return JSON.parse(localStorage.getItem('pendingOfferUploads') ?? '[]');
}

function queuedIds() {
  return JSON.parse(localStorage.getItem('pendingOfferUploads') ?? '[]').map(
    (upload: { offerId: string }) => upload.offerId,
  );
}

function offerWithStatus(status: string) {
  return { offer: { status } };
}

describe('pending offer uploads', () => {
  beforeEach(() => {
    localStorage.clear();
    vi.resetAllMocks();
    uploadToDexie.mockResolvedValue(undefined);
    uploadToMintGarden.mockResolvedValue(undefined);
    getWallet.mockResolvedValue({ wallet: { fingerprint: 1 } });
  });

  it('waits while the offer is pending, then uploads it once', async () => {
    queue('a');

    getOffer.mockResolvedValue(offerWithStatus('pending'));
    await processPendingOfferUploads(1);
    expect(uploadToDexie).not.toHaveBeenCalled();
    expect(queuedIds()).toEqual(['a']);

    getOffer.mockResolvedValue(offerWithStatus('active'));
    await processPendingOfferUploads(1);
    await processPendingOfferUploads(1);

    expect(uploadToDexie).toHaveBeenCalledTimes(1);
    expect(uploadToDexie).toHaveBeenCalledWith('offer-a', true);
    expect(uploadToMintGarden).not.toHaveBeenCalled();
    expect(queuedIds()).toEqual([]);
  });

  it('forgets offers that can no longer be taken', async () => {
    queue('a');

    getOffer.mockResolvedValue(offerWithStatus('cancelled'));
    await processPendingOfferUploads(1);

    expect(uploadToDexie).not.toHaveBeenCalled();
    expect(queuedIds()).toEqual([]);
  });

  it('only processes offers of the given wallet', async () => {
    queue('a', 2);

    getOffer.mockResolvedValue(offerWithStatus('active'));
    await processPendingOfferUploads(1);

    expect(getOffer).not.toHaveBeenCalled();
    expect(queuedIds()).toEqual(['a']);
  });

  it('keeps a missing offer if another wallet was opened meanwhile', async () => {
    queue('a');

    getOffer.mockRejectedValue({ kind: 'not_found', reason: 'missing' });
    getWallet.mockResolvedValue({ wallet: { fingerprint: 2 } });
    await processPendingOfferUploads(1);
    expect(queuedIds()).toEqual(['a']);

    getWallet.mockResolvedValue({ wallet: { fingerprint: 1 } });
    await processPendingOfferUploads(1);
    expect(queuedIds()).toEqual([]);
  });

  it('retries failed uploads later, only to marketplaces that failed', async () => {
    queue('a', 1, ['dexie', 'mintgarden']);
    getOffer.mockResolvedValue(offerWithStatus('active'));
    uploadToDexie.mockRejectedValueOnce(new Error('offline'));

    await processPendingOfferUploads(1, 0);
    expect(uploadToDexie).toHaveBeenCalledTimes(1);
    expect(uploadToMintGarden).toHaveBeenCalledTimes(1);
    expect(queued()[0].marketplaceIds).toEqual(['dexie']);

    // It isn't tried again until the delay has passed.
    await processPendingOfferUploads(1, 1_000);
    expect(uploadToDexie).toHaveBeenCalledTimes(1);

    await processPendingOfferUploads(1, 60_000);
    expect(uploadToDexie).toHaveBeenCalledTimes(2);
    expect(uploadToMintGarden).toHaveBeenCalledTimes(1);
    expect(queuedIds()).toEqual([]);
  });

  it('gives up after failing several times in a row', async () => {
    queue('a');
    getOffer.mockResolvedValue(offerWithStatus('active'));
    uploadToDexie.mockRejectedValue(new Error('offline'));

    for (let attempt = 0; attempt < MAX_UPLOAD_FAILURES; attempt++) {
      expect(queuedIds()).toEqual(['a']);
      // Each attempt is made long after the previous one's retry delay.
      await processPendingOfferUploads(1, attempt * 1_000_000_000);
    }

    expect(uploadToDexie).toHaveBeenCalledTimes(MAX_UPLOAD_FAILURES);
    expect(queuedIds()).toEqual([]);
  });
});

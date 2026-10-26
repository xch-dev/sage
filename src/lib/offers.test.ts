import { beforeEach, describe, expect, it, vi } from 'vitest';

const getOffer = vi.fn();

vi.mock('@/bindings', () => ({
  commands: { getOffer: (...args: unknown[]) => getOffer(...args) },
}));

const { fetchOfferRecords } = await import('./offers');

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

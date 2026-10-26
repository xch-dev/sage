import { describe, expect, it, vi } from 'vitest';

vi.mock('@/bindings', () => ({ commands: {} }));

import type { HandleRecord, RecipientError } from '@/bindings';
import {
  handleChangedFrom,
  isValidHandle,
  useHandleChangeStore,
  withHandleConfirmation,
} from './handles';

function changed(address: string, previous: string): RecipientError {
  return {
    recipient: '@alice',
    kind: 'changed',
    reason: 'changed',
    handle: 'alice',
    address,
    previous_address: previous,
  };
}

function recipientError(recipients: RecipientError[]) {
  return { kind: 'recipient', reason: 'failed', recipients };
}

describe('handles', () => {
  it('validates handle syntax', () => {
    expect(isValidHandle('@alice')).toBe(true);
    expect(isValidHandle('@Alice')).toBe(false);
    expect(isValidHandle('@al')).toBe(false);
    expect(isValidHandle('alice')).toBe(false);
  });

  it('detects changes from the previous or last sent address', () => {
    const record = {
      address: 'b',
      previous_address: null,
      last_sent_address: null,
    } as HandleRecord;

    expect(handleChangedFrom(record)).toBe(null);
    expect(handleChangedFrom({ ...record, last_sent_address: 'b' })).toBe(null);
    expect(handleChangedFrom({ ...record, last_sent_address: 'a' })).toBe('a');
    expect(handleChangedFrom({ ...record, previous_address: 'c' })).toBe('c');
  });

  it('retries with the new address once the change is confirmed', async () => {
    const request = vi
      .fn()
      .mockRejectedValueOnce(recipientError([changed('b', 'a')]))
      .mockResolvedValueOnce('done');

    const result = withHandleConfirmation(request);

    await vi.waitFor(() =>
      expect(useHandleChangeStore.getState().changes).toEqual([
        { handle: 'alice', previousAddress: 'a', address: 'b' },
      ]),
    );
    useHandleChangeStore.getState().finish(true);

    await expect(result).resolves.toBe('done');
    expect(request).toHaveBeenNthCalledWith(1, []);
    expect(request).toHaveBeenNthCalledWith(2, [
      { handle: 'alice', address: 'b' },
    ]);
  });

  it('stops when the change is declined', async () => {
    const request = vi
      .fn()
      .mockRejectedValue(recipientError([changed('b', 'a')]));

    const result = withHandleConfirmation(request);

    await vi.waitFor(() =>
      expect(useHandleChangeStore.getState().changes).not.toBe(null),
    );
    useHandleChangeStore.getState().finish(false);

    await expect(result).rejects.toMatchObject({ kind: 'cancelled' });
    expect(request).toHaveBeenCalledTimes(1);
  });

  it('passes other errors through without asking', async () => {
    const error = recipientError([
      { ...changed('b', 'a'), kind: 'unavailable' },
    ]);
    const request = vi.fn().mockRejectedValue(error);

    await expect(withHandleConfirmation(request)).rejects.toBe(error);
    expect(useHandleChangeStore.getState().changes).toBe(null);
  });
});

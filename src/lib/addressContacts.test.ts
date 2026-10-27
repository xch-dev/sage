import { describe, expect, it, vi } from 'vitest';

vi.mock('@/bindings', () => ({ commands: {} }));

import type { AddressContactsRecord, ContactRecord } from '@/bindings';
import { bech32m } from 'bech32';
import {
  AddressContactCache,
  labelContacts,
  NO_CONTACTS,
} from './addressContacts';

function address(byte: number) {
  return bech32m.encode('txch', bech32m.toWords(new Uint8Array(32).fill(byte)));
}

const ALICE = address(1);
const BOB = address(2);

function contact(
  id: number,
  name: string,
  overrides: Partial<ContactRecord> = {},
): ContactRecord {
  return {
    id,
    name,
    note: null,
    kind: 'address',
    recipient: ALICE,
    handle: null,
    address: ALICE,
    resolution: null,
    created_at: 1,
    updated_at: 1,
    ...overrides,
  };
}

const SAVED = contact(1, 'Alice');
const HANDLE = contact(2, 'Ally', {
  kind: 'handle',
  recipient: '@ally',
  handle: 'ally',
});

function testCache(handlesEnabled = true) {
  const flushes: (() => void)[] = [];
  const lookup = vi.fn(
    async (addresses: string[]): Promise<AddressContactsRecord[]> =>
      addresses.map((requested) => ({
        address: requested,
        contacts: requested === ALICE ? [SAVED, HANDLE] : [],
      })),
  );

  const cache = new AddressContactCache(lookup, {
    schedule: (flush) => flushes.push(flush),
    handlesEnabled,
  });

  return {
    cache,
    lookup,
    async flush() {
      while (flushes.length > 0) flushes.shift()?.();
      await new Promise((resolve) => setTimeout(resolve, 0));
    },
  };
}

describe('address contact labels', () => {
  it('only labels with address contacts when XCHandles is disabled', () => {
    expect(labelContacts([SAVED, HANDLE], true)).toEqual([SAVED, HANDLE]);
    expect(labelContacts([SAVED, HANDLE], false)).toEqual([SAVED]);
  });

  it('looks up every shown address in one request', async () => {
    const { cache, lookup, flush } = testCache();

    cache.watch(ALICE);
    cache.watch(BOB);
    cache.watch(ALICE);
    await flush();

    expect(lookup).toHaveBeenCalledTimes(1);
    expect(lookup).toHaveBeenCalledWith([ALICE, BOB]);
    expect(cache.get(ALICE)).toEqual([SAVED, HANDLE]);
    expect(cache.get(BOB)).toBe(NO_CONTACTS);
  });

  it('drops handle contacts when XCHandles is disabled', async () => {
    const { cache, flush } = testCache(false);

    cache.watch(ALICE);
    await flush();

    expect(cache.get(ALICE)).toEqual([SAVED]);
  });

  it('looks shown addresses up again after contacts change', async () => {
    const { cache, lookup, flush } = testCache();

    cache.watch(ALICE);
    await flush();
    cache.refresh();
    await flush();

    expect(lookup).toHaveBeenCalledTimes(2);
  });
});

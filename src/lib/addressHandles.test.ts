import { describe, expect, it, vi } from 'vitest';

vi.mock('@/bindings', () => ({ commands: {} }));

import type { AddressHandlesRecord, HandleRecord } from '@/bindings';
import { bech32m } from 'bech32';
import {
  AddressHandleCache,
  addressPrefix,
  handleTrust,
  isLookupAddress,
  MAX_LOOKUP_ADDRESSES,
  sortHandles,
} from './addressHandles';

function address(byte: number, prefix = 'txch') {
  return bech32m.encode(prefix, bech32m.toWords(new Uint8Array(32).fill(byte)));
}

const ALICE = address(1);
const BOB = address(2);

function handle(
  name: string,
  resolvesTo: string,
  overrides: Partial<HandleRecord> = {},
): HandleRecord {
  return {
    handle: name,
    handle_hash: '00'.repeat(32),
    address: resolvesTo,
    status: 'verified',
    previous_address: null,
    last_sent_address: null,
    changed_from: null,
    cached: true,
    verification: 'verified',
    verification_reason: null,
    owner_launcher_id: '11'.repeat(32),
    resolved_launcher_id: '22'.repeat(32),
    expiration: 2_000_000_000,
    slot_coin_id: '33'.repeat(32),
    slot_confirmation_height: 10,
    resolved_coin_id: '44'.repeat(32),
    resolved_confirmation_height: 12,
    updated_height: 12,
    verified_height: 20,
    verified_timestamp: 1_800_000_000,
    last_verified_at: 1_800_000_000,
    ...overrides,
  };
}

/** A fake store of handle mappings, looked up the way the backend does. */
function fakeBackend(initial: HandleRecord[]) {
  let mappings = initial;

  const lookup = vi.fn(
    async (addresses: string[]): Promise<AddressHandlesRecord[]> =>
      addresses.map((requested) => ({
        address: requested,
        handles: mappings.filter((record) => record.address === requested),
      })),
  );

  return {
    lookup,
    set(next: HandleRecord[]) {
      mappings = next;
    },
  };
}

/** A cache whose flushes and clock are controlled by the test. */
function testCache(backend: ReturnType<typeof fakeBackend>) {
  let now = 0;
  const flushes: (() => void)[] = [];

  const cache = new AddressHandleCache(backend.lookup, {
    now: () => now,
    schedule: (flush) => flushes.push(flush),
    ttlMs: 1000,
  });

  return {
    cache,
    advance(ms: number) {
      now += ms;
    },
    async flush() {
      while (flushes.length > 0) flushes.shift()?.();
      // Lets the lookup's promise settle.
      await new Promise((resolve) => setTimeout(resolve, 0));
    },
  };
}

describe('address handle labels', () => {
  it('only looks up addresses on the current network', () => {
    expect(addressPrefix(ALICE)).toBe('txch');
    expect(isLookupAddress(ALICE, 'txch')).toBe(true);
    expect(isLookupAddress(address(1, 'xch'), 'txch')).toBe(false);
    expect(isLookupAddress('@alice', 'txch')).toBe(false);
    expect(isLookupAddress('ab'.repeat(32), 'txch')).toBe(false);
    expect(isLookupAddress(address(1, 'nft'), 'txch')).toBe(false);
    expect(isLookupAddress(`${ALICE.slice(0, -1)}x`, 'txch')).toBe(false);
  });

  it('orders several handles deterministically, most trustworthy first', () => {
    const records = [
      handle('zed', ALICE),
      handle('old', ALICE, { verification: 'outdated' }),
      handle('moved', ALICE, { status: 'changed', previous_address: BOB }),
      handle('amy', ALICE),
      handle('sent', ALICE, { changed_from: BOB }),
    ];

    const names = (list: HandleRecord[]) => list.map((record) => record.handle);

    expect(names(sortHandles(records))).toEqual([
      'amy',
      'zed',
      'old',
      'moved',
      'sent',
    ]);
    expect(names(sortHandles([...records].reverse()))).toEqual(
      names(sortHandles(records)),
    );
  });

  it('never labels with mappings that are stale, unverified or expired', () => {
    const records = [
      handle('stale', ALICE, { verification: 'stale' }),
      handle('unverified', ALICE, { verification: 'unverified' }),
      handle('expired', ALICE, { verification: 'expired' }),
      handle('ok', ALICE),
    ];

    expect(sortHandles(records).map((record) => record.handle)).toEqual(['ok']);
  });

  it('distinguishes verified, outdated and changed mappings', () => {
    expect(handleTrust(handle('a', ALICE))).toBe('verified');
    expect(handleTrust(handle('a', ALICE, { verification: 'outdated' }))).toBe(
      'outdated',
    );
    expect(
      handleTrust(
        handle('a', ALICE, { status: 'changed', previous_address: BOB }),
      ),
    ).toBe('changed');
    expect(handleTrust(handle('a', ALICE, { changed_from: BOB }))).toBe(
      'changed',
    );
    // A change is the more important warning when a mapping is also outdated.
    expect(
      handleTrust(
        handle('a', ALICE, { verification: 'outdated', changed_from: BOB }),
      ),
    ).toBe('changed');
  });

  it('batches every address shown together into one request', async () => {
    const backend = fakeBackend([handle('alice', ALICE)]);
    const { cache, flush } = testCache(backend);

    // A page of rows, several showing the same address.
    for (const value of [ALICE, BOB, ALICE, ALICE, BOB]) cache.watch(value);
    await flush();

    expect(backend.lookup).toHaveBeenCalledTimes(1);
    expect(backend.lookup).toHaveBeenCalledWith([ALICE, BOB]);
    expect(cache.get(ALICE)?.map((record) => record.handle)).toEqual(['alice']);
    expect(cache.get(BOB)).toEqual([]);
  });

  it('splits batches at the backend limit', async () => {
    const backend = fakeBackend([]);
    const { cache, flush } = testCache(backend);

    const count = MAX_LOOKUP_ADDRESSES + 5;
    for (let i = 0; i < count; i++) cache.watch(`address-${i}`);
    await flush();

    expect(backend.lookup).toHaveBeenCalledTimes(2);
    expect(backend.lookup.mock.calls[0][0]).toHaveLength(MAX_LOOKUP_ADDRESSES);
    expect(backend.lookup.mock.calls[1][0]).toHaveLength(5);
  });

  it('reuses lookups until they expire', async () => {
    const backend = fakeBackend([handle('alice', ALICE)]);
    const { cache, flush, advance } = testCache(backend);

    const unwatch = cache.watch(ALICE);
    await flush();
    unwatch();

    cache.watch(ALICE);
    await flush();
    expect(backend.lookup).toHaveBeenCalledTimes(1);

    advance(1001);
    cache.watch(ALICE);
    await flush();
    expect(backend.lookup).toHaveBeenCalledTimes(2);
  });

  it('notifies subscribers when labels arrive', async () => {
    const backend = fakeBackend([handle('alice', ALICE)]);
    const { cache, flush } = testCache(backend);
    const listener = vi.fn();
    cache.subscribe(listener);

    cache.watch(ALICE);
    expect(cache.get(ALICE)).toBeUndefined();
    await flush();

    expect(listener).toHaveBeenCalled();
    expect(cache.get(ALICE)).toHaveLength(1);
  });

  it('moves the label when the resolved NFT is transferred', async () => {
    const backend = fakeBackend([handle('alice', ALICE)]);
    const { cache, flush } = testCache(backend);

    cache.watch(ALICE);
    cache.watch(BOB);
    await flush();
    expect(cache.get(ALICE)?.map((record) => record.handle)).toEqual(['alice']);

    // The handle's NFT moved to BOB, and resolving it again recorded that.
    backend.set([
      handle('alice', BOB, { status: 'changed', previous_address: ALICE }),
    ]);
    cache.refresh();
    await flush();

    expect(backend.lookup).toHaveBeenCalledTimes(2);
    expect(cache.get(ALICE)).toEqual([]);
    const [moved] = cache.get(BOB) ?? [];
    expect(moved.handle).toBe('alice');
    expect(handleTrust(moved)).toBe('changed');
  });

  it('shows outdated mappings as outdated, not verified', async () => {
    const backend = fakeBackend([
      handle('alice', ALICE, { verification: 'outdated' }),
    ]);
    const { cache, flush } = testCache(backend);

    cache.watch(ALICE);
    await flush();

    const [record] = cache.get(ALICE) ?? [];
    expect(handleTrust(record)).toBe('outdated');
  });

  it('shows no labels if the lookup fails', async () => {
    const backend = fakeBackend([]);
    backend.lookup.mockRejectedValueOnce(new Error('unsupported network'));
    const warn = vi.spyOn(console, 'warn').mockImplementation(vi.fn());
    const { cache, flush } = testCache(backend);

    cache.watch(ALICE);
    await flush();

    expect(cache.get(ALICE)).toEqual([]);
    warn.mockRestore();
  });

  it('drops lookups from before the cache was cleared', async () => {
    const backend = fakeBackend([handle('alice', ALICE)]);
    const { cache, flush } = testCache(backend);

    let release: (() => void) | undefined;
    backend.lookup.mockImplementationOnce(
      (addresses) =>
        new Promise((resolve) => {
          release = () =>
            resolve(
              addresses.map((requested) => ({
                address: requested,
                handles: [handle('previous-wallet', requested)],
              })),
            );
        }),
    );

    cache.watch(ALICE);
    await flush();

    // Switching wallets clears the cache and looks the address up again.
    cache.clear();
    await flush();
    release?.();
    await flush();

    expect(backend.lookup).toHaveBeenCalledTimes(2);
    expect(cache.get(ALICE)?.map((record) => record.handle)).toEqual(['alice']);
  });
});

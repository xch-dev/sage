import { AddressHandlesRecord, commands, HandleRecord } from '@/bindings';
import { bech32m } from 'bech32';

/** The most addresses `lookup_address_handles` accepts in one request. */
export const MAX_LOOKUP_ADDRESSES = 1000;

/**
 * How long a lookup is reused before it's made again. The lookup only reads
 * the local store, so this just limits how often the same page asks for it.
 */
const CACHE_TTL_MS = 60_000;

/**
 * How trustworthy a known handle is as a label for the address it resolves to.
 *
 * - `verified`: verified against the chain within the last day, and it still
 *   resolves where the user last knew it to.
 * - `outdated`: last verified more than a day ago, so it may have moved since.
 * - `changed`: it resolves here now, but resolved somewhere else when the user
 *   last used it, so it may belong to someone else.
 */
export type HandleTrust = 'verified' | 'outdated' | 'changed';

export function handleTrust(record: HandleRecord): HandleTrust {
  if (record.status === 'changed' || record.changed_from) return 'changed';
  if (record.verification !== 'verified') return 'outdated';
  return 'verified';
}

const TRUST_ORDER: Record<HandleTrust, number> = {
  verified: 0,
  outdated: 1,
  changed: 2,
};

/**
 * Keeps only the handles which may label an address, most trustworthy first
 * and then by name, so the same address is always labelled the same way.
 *
 * The backend only returns verified and outdated mappings, but anything else
 * is dropped here too, so that it can never be shown as a label.
 */
export function sortHandles(records: HandleRecord[]): HandleRecord[] {
  return records
    .filter(
      (record) =>
        record.verification === 'verified' ||
        record.verification === 'outdated',
    )
    .sort(
      (a, b) =>
        TRUST_ORDER[handleTrust(a)] - TRUST_ORDER[handleTrust(b)] ||
        (a.handle < b.handle ? -1 : a.handle > b.handle ? 1 : 0),
    );
}

/** The bech32m prefix of an address, or `null` if it isn't an address. */
export function addressPrefix(address: string): string | null {
  try {
    const decoded = bech32m.decode(address, 100);
    return bech32m.fromWords(decoded.words).length === 32
      ? decoded.prefix
      : null;
  } catch {
    return null;
  }
}

/**
 * Whether a value is an address on the current network, so it's worth looking
 * up. Anything else, like coin IDs, launcher IDs or handles, is never looked
 * up, and a single invalid address would fail the whole batch.
 */
export function isLookupAddress(value: string, prefix: string): boolean {
  return addressPrefix(value) === prefix;
}

interface Entry {
  handles: HandleRecord[];
  expiresAt: number;
}

export type LookupAddresses = (
  addresses: string[],
) => Promise<AddressHandlesRecord[]>;

export interface AddressHandleCacheOptions {
  now?: () => number;
  /** Runs a flush once everything requested in the current render is queued. */
  schedule?: (flush: () => void) => void;
  ttlMs?: number;
}

/** Shared by every address without handles, so it's a stable value. */
export const NO_HANDLES: HandleRecord[] = [];

/**
 * A client-side cache of the known handles for each address.
 *
 * Components watch the addresses they show. Every address watched before the
 * next flush is sent in one deduplicated `lookup_address_handles` request, so
 * a page makes one request however many addresses it shows. The lookup only
 * reads the local store, so it never causes a network request for an address.
 */
export class AddressHandleCache {
  private entries = new Map<string, Entry>();
  private watchers = new Map<string, number>();
  private queued = new Set<string>();
  /** The latest request for each address whose result hasn't arrived yet. */
  private pending = new Map<string, number>();
  private requests = 0;
  /** Set once the backend says this network has no XCHandles registry. */
  private unsupported = false;
  private listeners = new Set<() => void>();
  private generation = 0;
  private scheduled = false;

  private readonly now: () => number;
  private readonly schedule: (flush: () => void) => void;
  private readonly ttlMs: number;

  constructor(
    private readonly lookup: LookupAddresses,
    options: AddressHandleCacheOptions = {},
  ) {
    this.now = options.now ?? Date.now;
    this.schedule = options.schedule ?? ((flush) => setTimeout(flush, 0));
    this.ttlMs = options.ttlMs ?? CACHE_TTL_MS;
  }

  /** The known handles for an address, or `undefined` if not looked up yet. */
  get(address: string): HandleRecord[] | undefined {
    return this.entries.get(address)?.handles;
  }

  subscribe = (listener: () => void): (() => void) => {
    this.listeners.add(listener);
    return () => {
      this.listeners.delete(listener);
    };
  };

  /**
   * Marks an address as shown, and looks it up unless it was recently. Returns
   * a function to call once it's no longer shown.
   */
  watch(address: string): () => void {
    this.watchers.set(address, (this.watchers.get(address) ?? 0) + 1);
    this.request(address);

    return () => {
      const count = (this.watchers.get(address) ?? 1) - 1;
      if (count > 0) this.watchers.set(address, count);
      else this.watchers.delete(address);
    };
  }

  /**
   * Looks up every shown address again, keeping the current labels until the
   * new ones arrive. Used after handles may have been verified or changed.
   */
  refresh() {
    for (const entry of this.entries.values()) entry.expiresAt = 0;
    for (const address of this.watchers.keys()) this.request(address, true);
  }

  /** Forgets everything, for example when switching wallets. */
  clear() {
    this.generation += 1;
    this.entries.clear();
    this.queued.clear();
    this.pending.clear();
    this.unsupported = false;
    this.notify();
    for (const address of this.watchers.keys()) this.request(address);
  }

  private request(address: string, force = false) {
    if (this.unsupported) return;

    const entry = this.entries.get(address);
    if (!force && entry && entry.expiresAt > this.now()) return;
    if (this.queued.has(address)) return;
    if (!force && this.pending.has(address)) return;

    this.queued.add(address);

    if (!this.scheduled) {
      this.scheduled = true;
      this.schedule(() => this.flush());
    }
  }

  private flush() {
    this.scheduled = false;

    const addresses = [...this.queued];
    this.queued.clear();
    this.prune();

    for (let i = 0; i < addresses.length; i += MAX_LOOKUP_ADDRESSES) {
      this.send(addresses.slice(i, i + MAX_LOOKUP_ADDRESSES));
    }
  }

  private send(addresses: string[]) {
    const generation = this.generation;
    const id = ++this.requests;
    for (const address of addresses) this.pending.set(address, id);

    // Only stores results for addresses this is still the latest request for,
    // so an older lookup which finishes last can't replace newer labels.
    const store = (results: Map<string, HandleRecord[]> | null) => {
      if (generation !== this.generation) return;

      const expiresAt = this.now() + this.ttlMs;

      for (const address of addresses) {
        if (this.pending.get(address) !== id) continue;
        this.pending.delete(address);

        // A failed lookup keeps the labels already shown, if there are any.
        const handles = results
          ? sortHandles(results.get(address) ?? [])
          : (this.entries.get(address)?.handles ?? []);

        this.entries.set(address, {
          handles: handles.length === 0 ? NO_HANDLES : handles,
          expiresAt,
        });
      }

      this.notify();
    };

    this.lookup(addresses)
      .then((records) => {
        store(
          new Map(records.map((record) => [record.address, record.handles])),
        );
      })
      .catch((error: unknown) => {
        if (isUnsupportedNetwork(error)) {
          // Nothing will ever be found, so stop asking until the network or
          // wallet changes.
          if (generation === this.generation) this.unsupported = true;
        } else {
          // Labels are only a convenience, so a failed lookup shows no new
          // labels rather than an error, and isn't retried until it expires.
          console.warn('Failed to look up known handles', error);
        }
        store(null);
      });
  }

  /** Forgets expired lookups for addresses which aren't shown anymore. */
  private prune() {
    const now = this.now();
    for (const [address, entry] of this.entries) {
      if (entry.expiresAt <= now && !this.watchers.has(address)) {
        this.entries.delete(address);
      }
    }
  }

  private notify() {
    for (const listener of this.listeners) listener();
  }
}

function isUnsupportedNetwork(error: unknown): boolean {
  return (
    typeof error === 'object' &&
    error !== null &&
    'handle_error_code' in error &&
    error.handle_error_code === 'unsupported_network'
  );
}

export const addressHandles = new AddressHandleCache(async (addresses) => {
  const response = await commands.lookupAddressHandles({ addresses });
  return response.results;
});

/** How often the handles owned by a wallet are verified again. */
const OWNED_REFRESH_INTERVAL_MS = 12 * 60 * 60 * 1000;

function ownedRefreshKey(fingerprint: number, prefix: string) {
  return `xchandles-owned-refresh-${fingerprint}-${prefix}`;
}

/**
 * Verifies the handles this wallet's handle NFTs name, so the wallet's own
 * addresses are labelled with the handles it owns. The NFTs' display names are
 * only used by the backend to pick which handles to resolve, and each one is
 * verified against the registry before it's stored.
 *
 * This makes requests to the XCHandles API, and other backend requests wait
 * for it to finish (for up to 30 seconds), so it's only done every 12 hours
 * for each wallet and network.
 */
export async function refreshOwnedHandles(fingerprint: number, prefix: string) {
  const key = ownedRefreshKey(fingerprint, prefix);
  const last = Number(localStorage.getItem(key) ?? 0);

  if (Date.now() - last < OWNED_REFRESH_INTERVAL_MS) return;

  try {
    await commands.getHandles({
      filter: 'owned',
      include_unverified: false,
      refresh: true,
      offset: 0,
      limit: 1,
    });
  } catch (error) {
    // Networks without XCHandles have nothing to refresh, so there's no point
    // trying again each time the wallet is opened.
    if (!isUnsupportedNetwork(error)) throw error;
    localStorage.setItem(key, String(Date.now()));
    return;
  }

  // Only recorded once it succeeds, so a failed refresh is tried again the
  // next time the wallet is opened.
  localStorage.setItem(key, String(Date.now()));
  addressHandles.refresh();
}

import { bech32m } from 'bech32';

/**
 * The most addresses `lookup_address_handles` and `lookup_address_contacts`
 * accept in one request.
 */
export const MAX_LOOKUP_ADDRESSES = 1000;

/**
 * How long a lookup is reused before it's made again. Lookups only read the
 * local store, so this just limits how often the same page asks for them.
 */
const CACHE_TTL_MS = 60_000;

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

interface Entry<T> {
  values: T[];
  expiresAt: number;
}

export interface AddressLookupCacheOptions {
  now?: () => number;
  /** Runs a flush once everything requested in the current render is queued. */
  schedule?: (flush: () => void) => void;
  ttlMs?: number;
}

export interface AddressLookup<T> {
  /** Looks up a batch of addresses, returning what was found for each one. */
  lookup: (addresses: string[]) => Promise<Map<string, T[]>>;
  /** Filters and orders what was found for an address before it's stored. */
  normalize?: (values: T[]) => T[];
  /** Shared by every address with nothing found, so it's a stable value. */
  empty: T[];
  /** Whether an error means nothing will ever be found on this network. */
  isUnsupported?: (error: unknown) => boolean;
  /** Describes what's looked up, for warnings about failed lookups. */
  description: string;
}

/**
 * A client-side cache of what's known about each address, such as the handles
 * which resolve to it, or the contacts it's saved as.
 *
 * Components watch the addresses they show. Every address watched before the
 * next flush is sent in one deduplicated request, so a page makes one request
 * however many addresses it shows. Lookups only read the local store, so they
 * never cause a network request for an address.
 */
export class AddressLookupCache<T> {
  private entries = new Map<string, Entry<T>>();
  private watchers = new Map<string, number>();
  private queued = new Set<string>();
  /** The latest request for each address whose result hasn't arrived yet. */
  private pending = new Map<string, number>();
  private requests = 0;
  /** Set once the backend says this network doesn't support the lookup. */
  private unsupported = false;
  private listeners = new Set<() => void>();
  private generation = 0;
  private scheduled = false;

  private readonly now: () => number;
  private readonly schedule: (flush: () => void) => void;
  private readonly ttlMs: number;

  constructor(
    private readonly source: AddressLookup<T>,
    options: AddressLookupCacheOptions = {},
  ) {
    this.now = options.now ?? Date.now;
    this.schedule = options.schedule ?? ((flush) => setTimeout(flush, 0));
    this.ttlMs = options.ttlMs ?? CACHE_TTL_MS;
  }

  /** What's known about an address, or `undefined` if not looked up yet. */
  get(address: string): T[] | undefined {
    return this.entries.get(address)?.values;
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
   * new ones arrive. Used after what's known about them may have changed.
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
    const store = (results: Map<string, T[]> | null) => {
      if (generation !== this.generation) return;

      const expiresAt = this.now() + this.ttlMs;
      const normalize = this.source.normalize ?? ((values: T[]) => values);

      for (const address of addresses) {
        if (this.pending.get(address) !== id) continue;
        this.pending.delete(address);

        // A failed lookup keeps the labels already shown, if there are any.
        const values = results
          ? normalize(results.get(address) ?? [])
          : (this.entries.get(address)?.values ?? []);

        this.entries.set(address, {
          values: values.length === 0 ? this.source.empty : values,
          expiresAt,
        });
      }

      this.notify();
    };

    this.source
      .lookup(addresses)
      .then(store)
      .catch((error: unknown) => {
        if (this.source.isUnsupported?.(error)) {
          // Nothing will ever be found, so stop asking until the network or
          // wallet changes.
          if (generation === this.generation) this.unsupported = true;
        } else {
          // Labels are only a convenience, so a failed lookup shows no new
          // labels rather than an error, and isn't retried until it expires.
          console.warn(`Failed to look up ${this.source.description}`, error);
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

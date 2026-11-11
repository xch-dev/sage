import { AddressHandlesRecord, commands, HandleRecord } from '@/bindings';
import { AddressLookupCache, AddressLookupCacheOptions } from './addressLookup';

export {
  addressPrefix,
  isLookupAddress,
  MAX_LOOKUP_ADDRESSES,
} from './addressLookup';

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

export type LookupAddresses = (
  addresses: string[],
) => Promise<AddressHandlesRecord[]>;

/** Shared by every address without handles, so it's a stable value. */
export const NO_HANDLES: HandleRecord[] = [];

/**
 * A client-side cache of the known handles for each address, looked up with
 * `lookup_address_handles`. Handles which may not label an address are dropped,
 * and the rest are ordered with `sortHandles`.
 */
export class AddressHandleCache extends AddressLookupCache<HandleRecord> {
  constructor(
    lookup: LookupAddresses,
    options: AddressLookupCacheOptions = {},
  ) {
    super(
      {
        lookup: async (addresses) =>
          new Map(
            (await lookup(addresses)).map((record) => [
              record.address,
              record.handles,
            ]),
          ),
        normalize: sortHandles,
        empty: NO_HANDLES,
        isUnsupported: isUnsupportedNetwork,
        description: 'known handles',
      },
      options,
    );
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

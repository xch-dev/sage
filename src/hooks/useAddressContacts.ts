import { ContactRecord } from '@/bindings';
import { useAddressPrefix } from '@/hooks/useAddressHandles';
import { addressContacts, NO_CONTACTS } from '@/lib/addressContacts';
import { isLookupAddress } from '@/lib/addressLookup';
import { useEffect, useMemo, useSyncExternalStore } from 'react';

/**
 * The contacts an address is saved as, sorted by name. Every address shown in
 * the same render is looked up together in one request, and values which
 * aren't addresses on the current network are never looked up.
 *
 * Unlike handles, contacts are looked up even in builds without XCHandles,
 * since raw-address contacts are still available there.
 */
export function useAddressContacts(
  address: string | null | undefined,
): ContactRecord[] {
  const prefix = useAddressPrefix();
  const value = address?.trim().toLowerCase();

  const key = useMemo(
    () => (value && prefix && isLookupAddress(value, prefix) ? value : null),
    [value, prefix],
  );

  useEffect(() => (key ? addressContacts.watch(key) : undefined), [key]);

  const contacts = useSyncExternalStore(addressContacts.subscribe, () =>
    key ? addressContacts.get(key) : undefined,
  );

  return contacts ?? NO_CONTACTS;
}

/** Forgets contact labels when the wallet or network changes. */
export function useAddressContactsLifecycle(fingerprint: number | null) {
  const prefix = useAddressPrefix();

  useEffect(() => {
    addressContacts.clear();
  }, [fingerprint, prefix]);
}

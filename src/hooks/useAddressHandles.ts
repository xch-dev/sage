import { HandleRecord } from '@/bindings';
import {
  addressHandles,
  addressPrefix,
  isLookupAddress,
  NO_HANDLES,
  refreshOwnedHandles,
} from '@/lib/addressHandles';
import { xchandlesEnabled } from '@/lib/features';
import { useWalletState } from '@/state';
import { useEffect, useSyncExternalStore } from 'react';

/** The address prefix of the current network, once the wallet is synced. */
export function useAddressPrefix(): string | null {
  return useWalletState((state) => addressPrefix(state.sync.receive_address));
}

/**
 * The known handles for an address, most trustworthy first. Every address
 * shown in the same render is looked up together in one request, and values
 * which aren't addresses on the current network are never looked up.
 */
export function useAddressHandles(
  address: string | null | undefined,
): HandleRecord[] {
  const prefix = useAddressPrefix();
  const value = address?.trim().toLowerCase();

  const key =
    xchandlesEnabled && value && prefix && isLookupAddress(value, prefix)
      ? value
      : null;

  useEffect(() => (key ? addressHandles.watch(key) : undefined), [key]);

  const handles = useSyncExternalStore(addressHandles.subscribe, () =>
    key ? addressHandles.get(key) : undefined,
  );

  return handles ?? NO_HANDLES;
}

/** How long after opening a wallet its owned handles are verified. */
const OWNED_REFRESH_DELAY_MS = 10_000;

/**
 * Forgets known handles when the wallet or network changes, and verifies the
 * handles the wallet owns in the background shortly after it's opened.
 */
export function useAddressHandlesLifecycle(fingerprint: number | null) {
  const prefix = useAddressPrefix();

  useEffect(() => {
    addressHandles.clear();
  }, [fingerprint, prefix]);

  useEffect(() => {
    if (!xchandlesEnabled || fingerprint === null || !prefix) return;

    const timeout = setTimeout(() => {
      refreshOwnedHandles(fingerprint, prefix).catch((error) => {
        // Networks without XCHandles, or an unreachable API, just mean the
        // wallet's own addresses aren't labelled.
        console.warn('Failed to refresh owned handles', error);
      });
    }, OWNED_REFRESH_DELAY_MS);

    return () => clearTimeout(timeout);
  }, [fingerprint, prefix]);
}

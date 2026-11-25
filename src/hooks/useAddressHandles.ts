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
import { useEffect, useMemo, useSyncExternalStore } from 'react';

/** The address prefix of the current network, once the wallet is synced. */
export function useAddressPrefix(): string | null {
  // Selects the address itself, so it's only decoded again when it changes,
  // rather than on every wallet state update.
  const receiveAddress = useWalletState((state) => state.sync.receive_address);
  return useMemo(() => addressPrefix(receiveAddress), [receiveAddress]);
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

  const key = useMemo(
    () =>
      xchandlesEnabled && value && prefix && isLookupAddress(value, prefix)
        ? value
        : null,
    [value, prefix],
  );

  useEffect(() => (key ? addressHandles.watch(key) : undefined), [key]);

  const handles = useSyncExternalStore(addressHandles.subscribe, () =>
    key ? addressHandles.get(key) : undefined,
  );

  return handles ?? NO_HANDLES;
}

/** How long after opening a wallet its owned handles are verified. */
const OWNED_REFRESH_DELAY_MS = 10_000;

/**
 * How often an open wallet checks whether its owned handles are due to be
 * verified again. `refreshOwnedHandles` decides whether they actually are.
 */
const OWNED_REFRESH_CHECK_MS = 60 * 60 * 1000;

/**
 * Forgets known handles when the wallet or network changes, and verifies the
 * handles the wallet owns in the background shortly after it's opened, and
 * again whenever they're due while it stays open.
 */
export function useAddressHandlesLifecycle(fingerprint: number | null) {
  const prefix = useAddressPrefix();

  useEffect(() => {
    addressHandles.clear();
  }, [fingerprint, prefix]);

  useEffect(() => {
    if (!xchandlesEnabled || fingerprint === null || !prefix) return;

    const refresh = () => {
      refreshOwnedHandles(fingerprint, prefix).catch((error) => {
        // An unreachable API just means the wallet's own addresses aren't
        // labelled until it's tried again.
        console.warn('Failed to refresh owned handles', error);
      });
    };

    const timeout = setTimeout(refresh, OWNED_REFRESH_DELAY_MS);
    // Wallets left open are verified again once their refresh is due.
    const interval = setInterval(refresh, OWNED_REFRESH_CHECK_MS);

    return () => {
      clearTimeout(timeout);
      clearInterval(interval);
    };
  }, [fingerprint, prefix]);
}

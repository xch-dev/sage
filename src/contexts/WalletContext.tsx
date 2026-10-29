import { KeyInfo, WalletInfo, commands } from '@/bindings';
import { CustomError } from '@/contexts/ErrorContext';
import { useAddressContactsLifecycle } from '@/hooks/useAddressContacts';
import { useAddressHandlesLifecycle } from '@/hooks/useAddressHandles';
import { useErrors } from '@/hooks/useErrors';
import { useColdWalletUnsigned } from '@/hooks/useColdWalletUnsigned';
import { fetchState, initializeWalletState } from '@/state';
import { createContext, useContext, useEffect, useState } from 'react';

interface WalletContextType {
  wallet: WalletInfo | null;
  setWallet: (wallet: WalletInfo | null) => void;
  isSwitching: boolean;
  setIsSwitching: (isSwitching: boolean) => void;
  /** True when the wallet has no signing keys (cold/watch-only wallet). */
  isReadOnly: boolean;
  /** True when the user has opted in to building unsigned transactions on cold wallets. */
  allowUnsigned: boolean;
  /** True when transaction-initiating UI should be disabled.
   *  Equivalent to `isReadOnly && !allowUnsigned`. */
  isTransactionDisabled: boolean;
}

export const WalletContext = createContext<WalletContextType | undefined>(
  undefined,
);

export function WalletProvider({ children }: { children: React.ReactNode }) {
  const [wallet, setWallet] = useState<WalletInfo | null>(null);
  const [isSwitching, setIsSwitching] = useState(false);
  const { addError } = useErrors();
  const { allowUnsigned } = useColdWalletUnsigned();

  const fingerprint = wallet?.fingerprint ?? null;
  const [loadedKey, setLoadedKey] = useState<{
    fingerprint: number;
    key: KeyInfo | null;
  } | null>(null);

  useEffect(() => {
    if (fingerprint === null) return;

    let cancelled = false;

    commands
      .getKey({ fingerprint })
      .then((data) => {
        if (!cancelled) setLoadedKey({ fingerprint, key: data.key });
      })
      .catch(addError);

    return () => {
      cancelled = true;
    };
  }, [fingerprint, addError]);

  const isKeyLoaded =
    loadedKey !== null && loadedKey.fingerprint === fingerprint;

  // While the wallet's key is loading, it isn't treated as read-only, so that the
  // read-only banner doesn't flash for hot wallets.
  const isReadOnly =
    wallet !== null && isKeyLoaded && loadedKey.key?.has_secrets !== true;
  const isTransactionDisabled = isReadOnly && !allowUnsigned;

  useAddressHandlesLifecycle(wallet?.fingerprint ?? null);
  useAddressContactsLifecycle(wallet?.fingerprint ?? null);

  useEffect(() => {
    const init = async () => {
      try {
        initializeWalletState(setWallet);
        const data = await commands.getWallet({});
        setWallet(data.wallet);
        await fetchState();
      } catch (error) {
        const customError = error as CustomError;
        // Don't add unauthorized errors - they're expected when not logged in
        if (customError.kind !== 'unauthorized') {
          addError(customError);
        }
      }
    };

    init();
  }, [addError]);

  return (
    <WalletContext.Provider
      value={{
        wallet,
        setWallet,
        isSwitching,
        setIsSwitching,
        isReadOnly,
        allowUnsigned,
        isTransactionDisabled,
      }}
    >
      {children}
    </WalletContext.Provider>
  );
}

export function useWallet() {
  const context = useContext(WalletContext);
  if (context === undefined) {
    throw new Error('useWallet must be used within a WalletProvider');
  }
  return context;
}

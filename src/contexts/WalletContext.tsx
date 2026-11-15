import { KeyInfo, WalletInfo, commands, events } from '@/bindings';
import { CustomError } from '@/contexts/ErrorContext';
import { useAddressContactsLifecycle } from '@/hooks/useAddressContacts';
import { useAddressHandlesLifecycle } from '@/hooks/useAddressHandles';
import { useErrors } from '@/hooks/useErrors';
import { useColdWalletUnsigned } from '@/hooks/useColdWalletUnsigned';
import { usePendingOfferUploadsLifecycle } from '@/hooks/usePendingOfferUploads';
import { vaultInfo, watchedAddresses } from '@/lib/walletKind';
import { fetchState, initializeWalletState } from '@/state';
import { createContext, useContext, useEffect, useState } from 'react';

interface WalletContextType {
  wallet: WalletInfo | null;
  setWallet: (wallet: WalletInfo | null) => void;
  isSwitching: boolean;
  setIsSwitching: (isSwitching: boolean) => void;
  /** True when the wallet has no signing keys (cold/watch-only wallet). */
  isReadOnly: boolean;
  /** True for watch wallets, which only know addresses, so they can't even build
   *  unsigned transactions. */
  isWatchOnly: boolean;
  /** True for vault wallets. They can spend once one of their custody keys is
   *  attached as a signer. */
  isVault: boolean;
  /** True when the user has opted in to building unsigned transactions on cold wallets. */
  allowUnsigned: boolean;
  /** True when transaction-initiating UI should be disabled. Cold wallets can
   *  still build unsigned transactions if `allowUnsigned` is set, but watch
   *  wallets never can. */
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

  const isWatchOnly = watchedAddresses(wallet) !== null;
  const isVault = vaultInfo(wallet) !== null;
  // Watch and vault wallets have no key of their own in the keychain.
  const fingerprint =
    isWatchOnly || isVault ? null : (wallet?.fingerprint ?? null);
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
      .catch((error) => {
        // Without its key, the wallet is treated as read-only rather than hot.
        if (!cancelled) setLoadedKey({ fingerprint, key: null });
        addError(error);
      });

    return () => {
      cancelled = true;
    };
  }, [fingerprint, addError]);

  const isKeyLoaded =
    loadedKey !== null && loadedKey.fingerprint === fingerprint;

  // A vault spends with its custody signers' keys, so it's read-only until one
  // is attached. The wallet info changes when a signer is attached.
  const vaultFingerprint = isVault ? (wallet?.fingerprint ?? null) : null;
  const vaultSigners = JSON.stringify(vaultInfo(wallet)?.signers ?? []);
  const [vaultCanSpend, setVaultCanSpend] = useState<{
    fingerprint: number;
    canSpend: boolean;
  } | null>(null);

  useEffect(() => {
    if (vaultFingerprint === null) return;

    let cancelled = false;

    const check = () =>
      commands
        .getVault({})
        .then((data) => {
          if (cancelled) return;
          setVaultCanSpend({
            fingerprint: vaultFingerprint,
            canSpend: data.vault.signers.some((signer) => signer.custody),
          });
        })
        .catch(() => {
          if (!cancelled) {
            setVaultCanSpend({
              fingerprint: vaultFingerprint,
              canSpend: false,
            });
          }
        });

    check();

    // The vault's config, which says which signers are custody members, can
    // arrive after the wallet opens, such as once a mint confirms.
    const unlisten = events.syncEvent.listen((event) => {
      const type = event.payload.type;
      if (type === 'coin_state' || type === 'puzzle_batch_synced') check();
    });

    return () => {
      cancelled = true;
      unlisten.then((u) => u());
    };
  }, [vaultFingerprint, vaultSigners]);

  const canVaultSpend =
    vaultCanSpend !== null &&
    vaultCanSpend.fingerprint === vaultFingerprint &&
    vaultCanSpend.canSpend;

  // While the wallet's key is loading, it isn't treated as read-only, so that the
  // read-only banner doesn't flash for hot wallets.
  const isReadOnly =
    wallet !== null &&
    (isWatchOnly ||
      (isVault && !canVaultSpend) ||
      (!isVault && isKeyLoaded && loadedKey.key?.has_secrets !== true));
  const isTransactionDisabled =
    isReadOnly && (isWatchOnly || isVault || !allowUnsigned);

  useAddressHandlesLifecycle(wallet?.fingerprint ?? null);
  useAddressContactsLifecycle(wallet?.fingerprint ?? null);
  usePendingOfferUploadsLifecycle(wallet?.fingerprint ?? null);

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
        isWatchOnly,
        isVault,
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

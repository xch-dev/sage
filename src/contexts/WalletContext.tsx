import { KeyInfo, commands } from '@/bindings';
import { CustomError } from '@/contexts/ErrorContext';
import { useErrors } from '@/hooks/useErrors';
import { useColdWalletUnsigned } from '@/hooks/useColdWalletUnsigned';
import { fetchState, initializeWalletState } from '@/state';
import { createContext, useContext, useEffect, useState } from 'react';

interface WalletContextType {
  wallet: KeyInfo | null;
  setWallet: (wallet: KeyInfo | null) => void;
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
  const [wallet, setWallet] = useState<KeyInfo | null>(null);
  const [isSwitching, setIsSwitching] = useState(false);
  const { addError } = useErrors();
  const { allowUnsigned } = useColdWalletUnsigned();

  const isReadOnly = wallet !== null && wallet.has_secrets === false;
  const isTransactionDisabled = isReadOnly && !allowUnsigned;

  useEffect(() => {
    const init = async () => {
      try {
        initializeWalletState(setWallet);
        const data = await commands.getKey({});
        setWallet(data.key);
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

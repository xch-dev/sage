import { KeyInfo, commands } from '@/bindings';
import { CustomError } from '@/contexts/ErrorContext';
import { useErrors } from '@/hooks/useErrors';
import { fetchState, initializeWalletState } from '@/state';
import { createContext, useContext, useEffect, useState } from 'react';

interface WalletContextType {
  wallet: KeyInfo | null;
  setWallet: (wallet: KeyInfo | null) => void;
  isSwitching: boolean;
  setIsSwitching: (isSwitching: boolean) => void;
  isReadOnly: boolean;
}

export const WalletContext = createContext<WalletContextType | undefined>(
  undefined,
);

export function WalletProvider({ children }: { children: React.ReactNode }) {
  const [wallet, setWallet] = useState<KeyInfo | null>(null);
  const [isSwitching, setIsSwitching] = useState(false);
  const { addError } = useErrors();

  const isReadOnly = wallet !== null && wallet.has_secrets === false;

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
      value={{ wallet, setWallet, isSwitching, setIsSwitching, isReadOnly }}
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

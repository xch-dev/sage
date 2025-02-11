import { KeyInfo, commands } from '@/bindings';
import { useErrors } from '@/hooks/useErrors';
import { createContext, useContext, useEffect, useState } from 'react';
import { initializeWalletState, fetchState } from '@/state';
import { CustomError } from '@/contexts/ErrorContext';

interface WalletContextType {
  wallet: KeyInfo | null;
  setWallet: (wallet: KeyInfo | null) => void;
}

export const WalletContext = createContext<WalletContextType | undefined>(
  undefined,
);

export function WalletProvider({ children }: { children: React.ReactNode }) {
  const [wallet, setWallet] = useState<KeyInfo | null>(null);
  const { addError } = useErrors();

  useEffect(() => {
    const init = async () => {
      try {
        initializeWalletState(setWallet);
        const data = await commands.getKey({});
        setWallet(data.key);
        await fetchState();
      } catch (error) {
        addError(error as CustomError);
      }
    };

    init();
  }, [addError]);

  return (
    <WalletContext.Provider value={{ wallet, setWallet }}>
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

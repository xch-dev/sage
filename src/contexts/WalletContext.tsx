import { KeyInfo, commands } from '@/bindings';
import { useErrors } from '@/hooks/useErrors';
import { createContext, useContext, useEffect, useState } from 'react';
import { initializeWalletState } from '@/state';

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
    initializeWalletState(setWallet);
    commands
      .getKey({})
      .then((data) => setWallet(data.key))
      .catch(addError);
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
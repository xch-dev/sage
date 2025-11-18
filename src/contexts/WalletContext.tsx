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
}

export const WalletContext = createContext<WalletContextType | undefined>(
  undefined,
);

export function WalletProvider({ children }: { children: React.ReactNode }) {
  const [wallet, setWallet] = useState<KeyInfo | null>(null);
  const [isSwitching, setIsSwitching] = useState(false);
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
    <WalletContext.Provider
      value={{ wallet, setWallet, isSwitching, setIsSwitching }}
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

import { WalletConnectContext } from '@/contexts/WalletConnectContext';
import { useContext } from 'react';

export function useWalletConnect() {
  const context = useContext(WalletConnectContext);
  if (context === undefined) {
    throw new Error(
      'useWalletConnect must be used within a WalletConnectProvider',
    );
  }
  return context;
}

import { useEffect, useState } from 'react';
import { commands, WalletInfo } from '../bindings';

export function useWallet() {
  const [wallet, setWallet] = useState<WalletInfo | null>(null);

  useEffect(() => {
    commands.activeWallet().then((wallet) => {
      if (wallet.status === 'ok' && wallet.data) {
        setWallet(wallet.data);
      }
    });
  }, []);

  return { wallet };
}

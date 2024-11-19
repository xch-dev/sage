import { useEffect, useState } from 'react';
import { commands, KeyInfo } from '../bindings';

export function useWallet(initialized: boolean) {
  const [wallet, setWallet] = useState<KeyInfo | null>(null);

  useEffect(() => {
    if (initialized) {
      commands.getKey({}).then((wallet) => {
        if (wallet.status === 'ok' && wallet.data) {
          setWallet(wallet.data.key);
        } else {
          setWallet(null);
        }
      });
    }
  }, [initialized]);

  return wallet;
}

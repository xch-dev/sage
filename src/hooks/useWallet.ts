import { useEffect, useState } from 'react';
import { commands, KeyInfo } from '../bindings';

export function useWallet() {
  const [wallet, setWallet] = useState<KeyInfo | null>(null);

  useEffect(() => {
    commands.getKey({}).then((result) => {
      if (result.status === 'ok' && result.data) {
        setWallet(result.data.key);
      }
    });
  }, []);

  return { wallet };
}

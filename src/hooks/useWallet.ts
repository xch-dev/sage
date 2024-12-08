import { useEffect, useState } from 'react';
import { commands, KeyInfo } from '../bindings';
import { useErrors } from './useErrors';

export function useWallet(initialized: boolean) {
  const { addError } = useErrors();

  const [wallet, setWallet] = useState<KeyInfo | null>(null);

  useEffect(() => {
    if (!initialized) return;

    commands
      .getKey({})
      .then((data) => setWallet(data.key))
      .catch(addError);
  }, [initialized, addError]);

  return wallet;
}

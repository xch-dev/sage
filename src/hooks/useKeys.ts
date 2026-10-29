import { commands, KeyInfo } from '@/bindings';
import { useErrors } from '@/hooks/useErrors';
import { useEffect, useState } from 'react';

/** The public information of every key in the keychain, by fingerprint, or `null`
 *  until it has loaded. A BLS wallet's key has the same fingerprint as the wallet. */
export function useKeys() {
  const { addError } = useErrors();
  const [keys, setKeys] = useState<Map<number, KeyInfo> | null>(null);

  useEffect(() => {
    commands
      .getKeys({})
      .then((data) =>
        setKeys(new Map(data.keys.map((key) => [key.fingerprint, key]))),
      )
      .catch(addError);
  }, [addError]);

  return keys;
}

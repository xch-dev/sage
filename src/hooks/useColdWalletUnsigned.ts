import { useLocalStorage } from 'usehooks-ts';

export function useColdWalletUnsigned() {
  const [allowUnsigned, setAllowUnsigned] = useLocalStorage<boolean>(
    'cold-wallet-allow-unsigned',
    false,
  );

  return { allowUnsigned, setAllowUnsigned };
}

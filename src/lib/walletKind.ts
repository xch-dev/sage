import { WalletInfo } from '@/bindings';

/** The addresses a watch wallet watches, or `null` for other kinds of wallets. */
export function watchedAddresses(wallet: WalletInfo | null): string[] | null {
  if (wallet === null || wallet.kind === 'bls') return null;
  return wallet.kind.watch.addresses;
}

/** Splits pasted text into addresses, which can be separated by whitespace or
 *  commas. */
export function splitAddresses(text: string): string[] {
  return text
    .split(/[\s,]+/)
    .map((address) => address.trim())
    .filter((address) => address.length > 0);
}

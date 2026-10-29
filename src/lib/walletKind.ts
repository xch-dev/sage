import { WalletInfo } from '@/bindings';

/** The addresses a watch wallet watches, or `null` for other kinds of wallets. */
export function watchedAddresses(wallet: WalletInfo | null): string[] | null {
  if (wallet === null || wallet.kind === 'bls' || !('watch' in wallet.kind)) {
    return null;
  }
  return wallet.kind.watch.addresses;
}

/** A vault wallet's launcher id and the fingerprints of the keys that sign for
 *  it, or `null` for other kinds of wallets. */
export function vaultInfo(
  wallet: WalletInfo | null,
): { launcherId: string; signers: number[] } | null {
  if (wallet === null || wallet.kind === 'bls' || !('vault' in wallet.kind)) {
    return null;
  }
  return {
    launcherId: wallet.kind.vault.launcher_id,
    signers: wallet.kind.vault.signers,
  };
}

/** Whether a wallet uses the keychain key with this fingerprint, as its own key
 *  or as a vault signer. */
export function usesKey(wallet: WalletInfo, fingerprint: number): boolean {
  if (wallet.kind === 'bls') return wallet.fingerprint === fingerprint;
  return vaultInfo(wallet)?.signers.includes(fingerprint) ?? false;
}

/** Splits pasted text into addresses, which can be separated by whitespace or
 *  commas. */
export function splitAddresses(text: string): string[] {
  return text
    .split(/[\s,]+/)
    .map((address) => address.trim())
    .filter((address) => address.length > 0);
}

import { KeyInfo, WalletInfo, WalletKind } from '@/bindings';

export type WalletKindName = 'bls' | 'vault' | 'watch';

/** How a wallet is labeled in wallet lists. BLS wallets are hot or cold depending
 *  on whether the keychain holds their secret key. */
export type WalletBadge = 'hot' | 'cold' | 'vault' | 'watch';

export function walletKindName(kind: WalletKind): WalletKindName {
  if (kind === 'bls') return 'bls';
  if ('vault' in kind) return 'vault';
  return 'watch';
}

/** `key` is the keychain entry of a BLS wallet, which has the same fingerprint. */
export function walletBadge(
  wallet: WalletInfo,
  key: KeyInfo | null | undefined,
): WalletBadge {
  const kind = walletKindName(wallet.kind);
  if (kind === 'bls') return key?.has_secrets ? 'hot' : 'cold';
  return kind;
}

/** Only BLS wallets derive addresses from their key. */
export function canDeriveAddresses(wallet: WalletInfo | null): boolean {
  return wallet !== null && walletKindName(wallet.kind) === 'bls';
}

/** Whether the wallet can sign transactions itself. Vault spending isn't supported
 *  yet, and watch wallets never hold keys. */
export function canSign(
  wallet: WalletInfo,
  key: KeyInfo | null | undefined,
): boolean {
  return walletKindName(wallet.kind) === 'bls' && key?.has_secrets === true;
}

/** Whether the wallet can build unsigned transactions for another device to sign.
 *  Watch wallets can't, because the puzzle reveals of their coins are unknown, and
 *  vault spends aren't supported yet. */
export function canBuildUnsigned(wallet: WalletInfo): boolean {
  return walletKindName(wallet.kind) === 'bls';
}

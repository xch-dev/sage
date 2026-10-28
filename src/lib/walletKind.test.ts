import { describe, expect, it } from 'vitest';
import { KeyInfo, WalletInfo, WalletKind } from '@/bindings';
import {
  canBuildUnsigned,
  canDeriveAddresses,
  canSign,
  walletBadge,
  walletKindName,
} from './walletKind';

function wallet(kind: WalletKind): WalletInfo {
  return {
    name: 'Wallet',
    fingerprint: 1,
    kind,
    network_id: 'mainnet',
    emoji: null,
  };
}

function key(has_secrets: boolean): KeyInfo {
  return { fingerprint: 1, public_key: 'ab', has_secrets };
}

const vault: WalletKind = { vault: { launcher_id: '01'.repeat(32) } };
const watch: WalletKind = { watch: { addresses: ['xch1'] } };

describe('wallet kinds', () => {
  it('names every kind', () => {
    expect(walletKindName('bls')).toBe('bls');
    expect(walletKindName(vault)).toBe('vault');
    expect(walletKindName(watch)).toBe('watch');
  });

  it('labels BLS wallets by whether their key has secrets', () => {
    expect(walletBadge(wallet('bls'), key(true))).toBe('hot');
    expect(walletBadge(wallet('bls'), key(false))).toBe('cold');
    expect(walletBadge(wallet('bls'), null)).toBe('cold');
    expect(walletBadge(wallet(vault), null)).toBe('vault');
    expect(walletBadge(wallet(watch), null)).toBe('watch');
  });

  it('only lets BLS wallets derive, sign or build unsigned spends', () => {
    expect(canDeriveAddresses(wallet('bls'))).toBe(true);
    expect(canDeriveAddresses(wallet(vault))).toBe(false);
    expect(canDeriveAddresses(wallet(watch))).toBe(false);
    expect(canDeriveAddresses(null)).toBe(false);

    expect(canSign(wallet('bls'), key(true))).toBe(true);
    expect(canSign(wallet('bls'), key(false))).toBe(false);
    expect(canSign(wallet('bls'), null)).toBe(false);
    expect(canSign(wallet(vault), key(true))).toBe(false);

    expect(canBuildUnsigned(wallet('bls'))).toBe(true);
    expect(canBuildUnsigned(wallet(vault))).toBe(false);
    expect(canBuildUnsigned(wallet(watch))).toBe(false);
  });
});

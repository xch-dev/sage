import { describe, expect, it } from 'vitest';
import { WalletInfo, WalletKind } from '@/bindings';
import {
  splitAddresses,
  usesKey,
  vaultInfo,
  watchedAddresses,
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

describe('wallet kinds', () => {
  it('only watch wallets have watched addresses', () => {
    expect(watchedAddresses(null)).toBeNull();
    expect(watchedAddresses(wallet('bls'))).toBeNull();
    expect(
      watchedAddresses(wallet({ watch: { addresses: ['xch1a', 'xch1b'] } })),
    ).toEqual(['xch1a', 'xch1b']);
    expect(
      watchedAddresses(wallet({ vault: { launcher_id: 'ab', signers: [2] } })),
    ).toBeNull();
  });

  it('only vault wallets have vault info', () => {
    const vault = wallet({ vault: { launcher_id: 'ab', signers: [2, 3] } });

    expect(vaultInfo(null)).toBeNull();
    expect(vaultInfo(wallet('bls'))).toBeNull();
    expect(vaultInfo(wallet({ watch: { addresses: ['xch1a'] } }))).toBeNull();
    expect(vaultInfo(vault)).toEqual({ launcherId: 'ab', signers: [2, 3] });

    expect(usesKey(wallet('bls'), 1)).toBe(true);
    expect(usesKey(wallet('bls'), 2)).toBe(false);
    expect(usesKey(vault, 1)).toBe(false);
    expect(usesKey(vault, 3)).toBe(true);
  });

  it('splits pasted addresses', () => {
    expect(splitAddresses(' xch1a,xch1b\n xch1c\t, ')).toEqual([
      'xch1a',
      'xch1b',
      'xch1c',
    ]);
    expect(splitAddresses('   ')).toEqual([]);
  });
});

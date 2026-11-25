import { describe, expect, it } from 'vitest';
import { WalletInfo, WalletKind } from '@/bindings';
import { splitAddresses, watchedAddresses } from './walletKind';

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

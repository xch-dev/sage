import { describe, expect, it } from 'vitest';
import { VaultSpendRecord } from '@/bindings';
import { isVaultSpendSignable } from './VaultSpendAlert';

function record(overrides: Partial<VaultSpendRecord>): VaultSpendRecord {
  return {
    launcher_id: 'vault1test',
    verified: true,
    error: null,
    custody_changed: false,
    melted: false,
    reserved_fee: 0,
    unaccounted_xch: 0,
    ...overrides,
  };
}

describe('isVaultSpendSignable', () => {
  it('signs transactions that do not spend the vault', () => {
    expect(isVaultSpendSignable(null)).toBe(true);
    expect(isVaultSpendSignable(undefined)).toBe(true);
  });

  it('signs verified spends that keep the custody', () => {
    expect(isVaultSpendSignable(record({}))).toBe(true);
  });

  it('refuses unverified, custody-changing, and melting spends', () => {
    expect(isVaultSpendSignable(record({ verified: false }))).toBe(false);
    expect(isVaultSpendSignable(record({ custody_changed: true }))).toBe(false);
    expect(isVaultSpendSignable(record({ melted: true }))).toBe(false);
  });
});

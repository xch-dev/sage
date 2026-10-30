import { describe, expect, it } from 'vitest';
import {
  EMPTY_TIMELOCK,
  timelockSeconds,
  vaultRecoveryKey,
} from './vaultRecovery';

describe('timelockSeconds', () => {
  it('is null when left empty', () => {
    expect(timelockSeconds(EMPTY_TIMELOCK)).toBeNull();
    expect(timelockSeconds({ days: ' ', hours: '', minutes: '' })).toBeNull();
  });

  it('adds up days, hours, and minutes', () => {
    expect(timelockSeconds({ days: '', hours: '12', minutes: '' })).toBe(
      43_200,
    );
    expect(timelockSeconds({ days: '1', hours: '2', minutes: '3' })).toBe(
      86_400 + 7200 + 180,
    );
  });

  it('rejects values that are not whole numbers, or add up to zero', () => {
    expect(
      timelockSeconds({ days: '1.5', hours: '', minutes: '' }),
    ).toBeUndefined();
    expect(
      timelockSeconds({ days: '-1', hours: '', minutes: '' }),
    ).toBeUndefined();
    expect(
      timelockSeconds({ days: '0', hours: '0', minutes: '' }),
    ).toBeUndefined();
  });
});

describe('vaultRecoveryKey', () => {
  it('is null without a key', () => {
    expect(
      vaultRecoveryKey('  ', { days: '1', hours: '', minutes: '' }),
    ).toBeNull();
  });

  it('trims the key and includes the timelock', () => {
    expect(vaultRecoveryKey(' ab ', EMPTY_TIMELOCK)).toEqual({
      key: 'ab',
      timelock: null,
    });
    expect(
      vaultRecoveryKey('ab', { days: '', hours: '', minutes: '5' }),
    ).toEqual({ key: 'ab', timelock: 300 });
  });

  it('is undefined if the timelock is invalid', () => {
    expect(
      vaultRecoveryKey('ab', { days: 'x', hours: '', minutes: '' }),
    ).toBeUndefined();
  });
});

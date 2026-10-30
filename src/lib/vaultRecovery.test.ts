import { describe, expect, it } from 'vitest';
import {
  EMPTY_RECOVERY,
  EMPTY_TIMELOCK,
  RecoveryInput,
  recoveryKeys,
  timelockSeconds,
  vaultRecoveryKeys,
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

describe('recoveryKeys', () => {
  it('takes one key per line, keeping the spaces in mnemonics', () => {
    expect(recoveryKeys(' ab \n\n  cd ef\n')).toEqual(['ab', 'cd ef']);
    expect(recoveryKeys('   ')).toEqual([]);
  });
});

describe('vaultRecoveryKeys', () => {
  const input = (
    keys: string,
    threshold = '',
    timelock = EMPTY_TIMELOCK,
  ): RecoveryInput => ({ keys, threshold, timelock });

  it('is null without keys', () => {
    expect(
      vaultRecoveryKeys(
        input('  ', '1', { days: '1', hours: '', minutes: '' }),
      ),
    ).toBeNull();
    expect(vaultRecoveryKeys(EMPTY_RECOVERY)).toBeNull();
  });

  it('includes the keys, threshold and timelock', () => {
    expect(vaultRecoveryKeys(input(' ab '))).toEqual({
      keys: ['ab'],
      threshold: null,
      timelock: null,
    });
    expect(
      vaultRecoveryKeys(
        input('ab\ncd', '2', { days: '', hours: '', minutes: '5' }),
      ),
    ).toEqual({ keys: ['ab', 'cd'], threshold: 2, timelock: 300 });
  });

  it('is undefined if the threshold or timelock is invalid', () => {
    expect(
      vaultRecoveryKeys(input('ab', '', { days: 'x', hours: '', minutes: '' })),
    ).toBeUndefined();
    expect(vaultRecoveryKeys(input('ab', '0'))).toBeUndefined();
    expect(vaultRecoveryKeys(input('ab\ncd', '3'))).toBeUndefined();
    expect(vaultRecoveryKeys(input('ab', '1.5'))).toBeUndefined();
  });
});

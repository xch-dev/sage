import { VaultRecoveryKey } from '@/bindings';

/** A recovery timelock as entered in days, hours, and minutes, as Chia Cloud
 *  Wallet sets it. Every part can be left empty. */
export interface TimelockInput {
  days: string;
  hours: string;
  minutes: string;
}

export const EMPTY_TIMELOCK: TimelockInput = {
  days: '',
  hours: '',
  minutes: '',
};

/** The timelock in seconds, or `null` if it's left empty, so that common
 *  timelocks are tried. Returns `undefined` if it isn't valid. */
export function timelockSeconds(
  timelock: TimelockInput,
): number | null | undefined {
  const parts = [
    [timelock.days, 86_400],
    [timelock.hours, 3600],
    [timelock.minutes, 60],
  ] as const;

  if (parts.every(([value]) => value.trim() === '')) {
    return null;
  }

  let seconds = 0;

  for (const [value, unit] of parts) {
    const trimmed = value.trim();
    if (trimmed === '') continue;
    if (!/^\d+$/.test(trimmed)) return undefined;
    seconds += Number(trimmed) * unit;
  }

  return seconds > 0 && Number.isSafeInteger(seconds) ? seconds : undefined;
}

/** The recovery key to rebuild a vault's config with, or `null` if no key was
 *  entered. Returns `undefined` if the timelock isn't valid. */
export function vaultRecoveryKey(
  key: string,
  timelock: TimelockInput,
): VaultRecoveryKey | null | undefined {
  const trimmed = key.trim();
  if (trimmed === '') return null;

  const seconds = timelockSeconds(timelock);
  if (seconds === undefined) return undefined;

  return { key: trimmed, timelock: seconds };
}

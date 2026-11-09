import { VaultRecoveryKeys } from '@/bindings';

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

/** The recovery keys, threshold, and timelock as entered. */
export interface RecoveryInput {
  keys: string;
  threshold: string;
  timelock: TimelockInput;
}

export const EMPTY_RECOVERY: RecoveryInput = {
  keys: '',
  threshold: '',
  timelock: EMPTY_TIMELOCK,
};

/** The recovery keys entered one per line, without empty or repeated lines.
 *  Mnemonics have spaces, so they can't be split on anything else. */
export function recoveryKeys(text: string): string[] {
  const keys = text
    .split('\n')
    .map((key) => key.trim())
    .filter((key) => key !== '');

  return [...new Set(keys)];
}

/** The recovery keys to rebuild a vault's config with, or `null` if no keys
 *  were entered, which is a vault without recovery. Returns `undefined` if the
 *  threshold or timelock isn't valid. */
export function vaultRecoveryKeys({
  keys,
  threshold,
  timelock,
}: RecoveryInput): VaultRecoveryKeys | null | undefined {
  const parsed = recoveryKeys(keys);
  if (parsed.length === 0) return null;

  // A lone key's threshold is always 1, and its field is hidden, so anything
  // left in it is ignored.
  const trimmed = parsed.length > 1 ? threshold.trim() : '';
  let parsedThreshold: number | null = null;

  if (trimmed !== '') {
    if (!/^\d+$/.test(trimmed)) return undefined;
    parsedThreshold = Number(trimmed);
    if (parsedThreshold < 1 || parsedThreshold > parsed.length) {
      return undefined;
    }
  }

  const seconds = timelockSeconds(timelock);
  if (seconds === undefined) return undefined;

  return { keys: parsed, threshold: parsedThreshold, timelock: seconds };
}

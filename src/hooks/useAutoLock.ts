import { useLocalStorage } from 'usehooks-ts';

/** How long, in minutes, the app can be idle before it logs out. 0 means never. */
export const AUTO_LOCK_OPTIONS = [0, 1, 5, 15, 30, 60];

export function useAutoLock() {
  const [storedMinutes, setMinutes] = useLocalStorage<number>(
    'sage-auto-lock-minutes',
    0,
  );

  const minutes = AUTO_LOCK_OPTIONS.includes(storedMinutes) ? storedMinutes : 0;

  return { minutes, setMinutes };
}

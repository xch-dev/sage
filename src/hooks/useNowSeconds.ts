import { nowSeconds } from '@/lib/expiration';
import { useEffect, useState } from 'react';

// The current unix time in seconds, refreshed at the start of every minute so
// expiration countdowns and validity checks don't go stale while a form is
// left open. Expirations are shown and picked to the minute, so this is
// frequent enough without re-rendering every second.
export function useNowSeconds(): number {
  const [now, setNow] = useState(nowSeconds);

  useEffect(() => {
    let timeout: ReturnType<typeof setTimeout>;
    const schedule = () => {
      timeout = setTimeout(
        () => {
          setNow(nowSeconds());
          schedule();
        },
        60_000 - (Date.now() % 60_000) + 50,
      );
    };
    schedule();
    return () => clearTimeout(timeout);
  }, []);

  return now;
}

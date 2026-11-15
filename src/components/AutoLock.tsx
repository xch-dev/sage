import { useAutoLock } from '@/hooks/useAutoLock';
import { logoutAndUpdateState } from '@/state';
import { useEffect } from 'react';
import { useNavigate } from 'react-router-dom';

const ACTIVITY_EVENTS = [
  'pointerdown',
  'pointermove',
  'keydown',
  'wheel',
  'touchstart',
] as const;

/** Logs out of the wallet after the app has been idle for the time set in Settings. */
export function AutoLock() {
  const { minutes } = useAutoLock();
  const navigate = useNavigate();

  useEffect(() => {
    if (minutes === 0) return;

    const limit = minutes * 60 * 1000;
    let lastActivity = Date.now();
    let locked = false;

    const onActivity = () => {
      lastActivity = Date.now();
    };

    // Timers don't run while a phone app is in the background, so the idle time is checked
    // against the clock instead of waiting for one timeout.
    const check = () => {
      if (locked || Date.now() - lastActivity < limit) return;
      locked = true;
      logoutAndUpdateState().then(() => navigate('/'));
    };

    const onVisibilityChange = () => {
      if (document.visibilityState === 'visible') check();
    };

    for (const event of ACTIVITY_EVENTS) {
      window.addEventListener(event, onActivity, { passive: true });
    }
    document.addEventListener('visibilitychange', onVisibilityChange);
    const interval = setInterval(check, 15 * 1000);

    return () => {
      for (const event of ACTIVITY_EVENTS) {
        window.removeEventListener(event, onActivity);
      }
      document.removeEventListener('visibilitychange', onVisibilityChange);
      clearInterval(interval);
    };
  }, [minutes, navigate]);

  return null;
}

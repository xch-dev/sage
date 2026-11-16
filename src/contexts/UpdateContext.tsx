import { hideUpdateToast, showUpdateToast } from '@/components/UpdateToast';
import {
  fetchLatestRelease,
  isNewerVersion,
  LatestRelease,
} from '@/lib/updates';
import { getVersion } from '@tauri-apps/api/app';
import { openUrl } from '@tauri-apps/plugin-opener';
import { platform } from '@tauri-apps/plugin-os';
import {
  createContext,
  ReactNode,
  useCallback,
  useContext,
  useEffect,
  useRef,
  useState,
} from 'react';
import { useLocalStorage } from 'usehooks-ts';

export const UPDATE_CHECKS_STORAGE_KEY = 'sage-update-checks';
/** The last version whose update toast was dismissed, so each release only pops up once. */
export const UPDATE_NOTIFIED_STORAGE_KEY = 'sage-update-notified-version';

const CHECK_INTERVAL_MS = 6 * 60 * 60 * 1000;
/** How soon to try again after a check fails, such as when Sage starts before the network is up. */
const RETRY_INTERVAL_MS = 15 * 60 * 1000;

export type UpdateStatus =
  | 'idle'
  | 'checking'
  | 'up-to-date'
  | 'available'
  | 'error';

export interface UpdateContextType {
  /** Whether this platform gets update notices. App stores update the mobile apps. */
  supported: boolean;
  /** Whether Sage checks for updates on its own. */
  enabled: boolean;
  setEnabled: (enabled: boolean) => void;
  currentVersion: string | null;
  status: UpdateStatus;
  /** The newer release, while one is available. */
  update: LatestRelease | null;
  checkNow: () => void;
  /** Opens the newer release's page, which also counts as having seen it, so its toast goes away. */
  openRelease: () => void;
}

const UpdateContext = createContext<UpdateContextType | undefined>(undefined);

const currentPlatform = platform();
const supported = currentPlatform !== 'ios' && currentPlatform !== 'android';

export function UpdateProvider({ children }: { children: ReactNode }) {
  const [enabled, setEnabledSetting] = useLocalStorage<boolean>(
    UPDATE_CHECKS_STORAGE_KEY,
    true,
  );
  const [currentVersion, setCurrentVersion] = useState<string | null>(null);
  const [status, setStatus] = useState<UpdateStatus>('idle');
  const [update, setUpdate] = useState<LatestRelease | null>(null);
  const [notifiedVersion, setNotifiedVersion] = useLocalStorage<string | null>(
    UPDATE_NOTIFIED_STORAGE_KEY,
    null,
  );
  // Bumped by Check Now while automatic checks are on, so the schedule restarts from that check and
  // a failure is retried like any other.
  const [restarts, setRestarts] = useState(0);
  // Only the most recent check's result is used, so one still running when checks are turned off
  // or another starts is ignored.
  const checkId = useRef(0);

  useEffect(() => {
    getVersion()
      .then(setCurrentVersion)
      .catch((error) => console.warn('Failed to read the app version', error));
  }, []);

  /** Returns whether the check failed. */
  const check = useCallback(async (version: string): Promise<boolean> => {
    const id = ++checkId.current;
    setStatus('checking');

    try {
      const latest = await fetchLatestRelease();
      if (id !== checkId.current) return false;

      if (isNewerVersion(latest.version, version)) {
        // Keep the same object while the version is unchanged, so later checks don't re-show the
        // toast.
        setUpdate((current) =>
          current?.version === latest.version ? current : latest,
        );
        setStatus('available');
      } else {
        setUpdate(null);
        setStatus('up-to-date');
      }
      return false;
    } catch (error) {
      if (id !== checkId.current) return false;
      console.warn('Failed to check for updates', error);
      setStatus('error');
      return true;
    }
  }, []);

  useEffect(() => {
    if (!supported || currentVersion === null) return;

    if (!enabled) {
      checkId.current++;
      setUpdate(null);
      setStatus('idle');
      return;
    }

    let cancelled = false;
    let timeout: ReturnType<typeof setTimeout> | undefined;

    const run = async () => {
      const failed = await check(currentVersion);
      if (cancelled) return;
      timeout = setTimeout(
        () => void run(),
        failed ? RETRY_INTERVAL_MS : CHECK_INTERVAL_MS,
      );
    };

    void run();

    return () => {
      cancelled = true;
      clearTimeout(timeout);
    };
  }, [enabled, currentVersion, check, restarts]);

  // Pop up a toast the first time each release is found. It stays until it's dismissed, and comes
  // back on the next start if it wasn't. The sidebar keeps showing the update after that.
  useEffect(() => {
    if (!update || update.version === notifiedVersion) return;

    const version = update.version;
    showUpdateToast(update, () => setNotifiedVersion(version));
    return () => hideUpdateToast(version);
  }, [update, notifiedVersion, setNotifiedVersion]);

  // Opening the release or turning checks off means the user knows about this version, so its
  // toast is closed and doesn't come back. A newer release still gets one.
  const markSeen = useCallback(() => {
    if (update) setNotifiedVersion(update.version);
  }, [update, setNotifiedVersion]);

  const openRelease = useCallback(() => {
    if (!update) return;
    void openUrl(update.url);
    markSeen();
  }, [update, markSeen]);

  const setEnabled = useCallback(
    (enabled: boolean) => {
      if (!enabled) markSeen();
      setEnabledSetting(enabled);
    },
    [markSeen, setEnabledSetting],
  );

  const checkNow = useCallback(() => {
    if (!supported || currentVersion === null) return;

    if (enabled) {
      setRestarts((restarts) => restarts + 1);
    } else {
      void check(currentVersion);
    }
  }, [enabled, currentVersion, check]);

  return (
    <UpdateContext.Provider
      value={{
        supported,
        enabled,
        setEnabled,
        currentVersion,
        status,
        update,
        checkNow,
        openRelease,
      }}
    >
      {children}
    </UpdateContext.Provider>
  );
}

export function useUpdates() {
  const context = useContext(UpdateContext);
  if (context === undefined) {
    throw new Error('useUpdates must be used within an UpdateProvider');
  }
  return context;
}

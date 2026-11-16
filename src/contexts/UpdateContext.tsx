import { hideUpdateToast, showUpdateToast } from '@/components/UpdateToast';
import {
  fetchLatestRelease,
  isNewerVersion,
  LatestRelease,
} from '@/lib/updates';
import { getVersion } from '@tauri-apps/api/app';
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
}

const UpdateContext = createContext<UpdateContextType | undefined>(undefined);

const currentPlatform = platform();
const supported = currentPlatform !== 'ios' && currentPlatform !== 'android';

export function UpdateProvider({ children }: { children: ReactNode }) {
  const [enabled, setEnabled] = useLocalStorage<boolean>(
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
  // Only the most recent check's result is used, so one still running when checks are turned off
  // or another starts is ignored.
  const checkId = useRef(0);

  useEffect(() => {
    getVersion()
      .then(setCurrentVersion)
      .catch((error) => console.warn('Failed to read the app version', error));
  }, []);

  const check = useCallback(async (version: string) => {
    const id = ++checkId.current;
    setStatus('checking');

    try {
      const latest = await fetchLatestRelease();
      if (id !== checkId.current) return;

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
    } catch (error) {
      if (id !== checkId.current) return;
      console.warn('Failed to check for updates', error);
      setStatus('error');
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

    void check(currentVersion);
    const interval = setInterval(
      () => void check(currentVersion),
      CHECK_INTERVAL_MS,
    );

    return () => clearInterval(interval);
  }, [enabled, currentVersion, check]);

  // Pop up a toast the first time each release is found. It stays until it's dismissed, and comes
  // back on the next start if it wasn't. The sidebar keeps showing the update after that.
  useEffect(() => {
    if (!update || update.version === notifiedVersion) return;

    const version = update.version;
    showUpdateToast(update, () => setNotifiedVersion(version));
    return () => hideUpdateToast(version);
  }, [update, notifiedVersion, setNotifiedVersion]);

  const checkNow = useCallback(() => {
    if (!supported || currentVersion === null) return;
    void check(currentVersion);
  }, [currentVersion, check]);

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

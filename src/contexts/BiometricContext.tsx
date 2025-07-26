import {
  authenticate,
  BiometryType,
  checkStatus,
} from '@tauri-apps/plugin-biometric';
import { platform } from '@tauri-apps/plugin-os';
import {
  createContext,
  ReactNode,
  useCallback,
  useEffect,
  useState,
} from 'react';
import { useLocalStorage } from 'usehooks-ts';

export interface BiometricContextType {
  enabled: boolean;
  available: boolean;
  promptIfEnabled: () => Promise<boolean>;
  enableIfAvailable: () => Promise<void>;
  disable: () => Promise<void>;
}

export const BiometricContext = createContext<BiometricContextType | undefined>(
  undefined,
);

const isMobile = platform() === 'ios' || platform() === 'android';

// It's unclear why this causes a crash if inside of the BiometricProvider useEffect,
// but it does - so moving it out here is a workaround for the issue until it's properly
// investigated.
const status = isMobile
  ? checkStatus()
  : Promise.resolve({ isAvailable: false, biometryType: BiometryType.None });

export function BiometricProvider({ children }: { children: ReactNode }) {
  const [enabled, setEnabled] = useLocalStorage('biometric', false);
  const [available, setAvailable] = useState(false);
  const [lastPrompt, setLastPrompt] = useState<number | null>(null);

  useEffect(() => {
    if (!isMobile) return;

    status.then((status) =>
      setAvailable(
        status.isAvailable && status.biometryType !== BiometryType.None,
      ),
    );
  }, []);

  const promptIfEnabled = useCallback(async () => {
    const now = performance.now();

    // Required every 5 minutes
    if (enabled && (lastPrompt === null || now - lastPrompt >= 1000 * 300)) {
      try {
        await authenticate('Authenticate with biometric', {
          allowDeviceCredential: false,
        });
        setLastPrompt(now);
        return true;
      } catch {
        return false;
      }
    }

    return true;
  }, [enabled, lastPrompt]);

  const enableIfAvailable = useCallback(async () => {
    if (!available) return;

    await authenticate('Enable biometric authentication');

    setEnabled(true);
  }, [available, setEnabled]);

  const disable = useCallback(async () => {
    if (available) {
      await authenticate('Disable biometric authentication');
    }

    setEnabled(false);
  }, [available, setEnabled]);

  return (
    <BiometricContext.Provider
      value={{
        enabled,
        available,
        promptIfEnabled,
        enableIfAvailable,
        disable,
      }}
    >
      {children}
    </BiometricContext.Provider>
  );
}

import { PasswordDialog } from '@/components/dialogs/PasswordDialog';
import { useBiometric } from '@/hooks/useBiometric';
import { useWallet } from '@/contexts/WalletContext';
import { platform } from '@tauri-apps/plugin-os';
import { createContext, ReactNode, useCallback, useRef, useState } from 'react';

const isMobile = platform() === 'ios' || platform() === 'android';

// Lazy-initialized keychain module (mobile only)
let keychainPromise: Promise<typeof import('tauri-plugin-keychain')> | null =
  null;
function getKeychain() {
  if (!isMobile) return null;
  if (!keychainPromise) keychainPromise = import('tauri-plugin-keychain');
  return keychainPromise;
}

async function keychainGet(key: string): Promise<string | null> {
  const mod = getKeychain();
  if (!mod) return null;
  try {
    const { getItem } = await mod;
    return await getItem(key);
  } catch {
    return null;
  }
}

async function keychainSave(key: string, password: string): Promise<void> {
  const mod = getKeychain();
  if (!mod) return;
  try {
    const { saveItem } = await mod;
    await saveItem(key, password);
  } catch {
    // Silently fail — keychain storage is best-effort
  }
}

async function keychainRemove(key: string): Promise<void> {
  const mod = getKeychain();
  if (!mod) return;
  try {
    const { removeItem } = await mod;
    await removeItem(key);
  } catch {
    // Silently fail
  }
}

// Biometric caching interval (5 minutes), matching existing BiometricContext behavior
const BIOMETRIC_CACHE_MS = 5 * 60 * 1000;

interface PasswordRequest {
  resolve: (password: string | null | undefined) => void;
  fingerprint: number | undefined;
}

export interface PasswordContextType {
  requestPassword: (hasPassword: boolean, fingerprint?: number) => Promise<string | null | undefined>;
  clearKeychainEntry: (fingerprint: number) => Promise<void>;
  updateKeychainEntry: (
    fingerprint: number,
    newPassword: string,
  ) => Promise<void>;
  clearAllKeychainEntries: (fingerprints: number[]) => Promise<void>;
}

export const PasswordContext = createContext<PasswordContextType | undefined>(
  undefined,
);

function keychainKey(fingerprint: number): string {
  return `sage-password-${fingerprint.toString()}`;
}

export function PasswordProvider({ children }: { children: ReactNode }) {
  const [open, setOpen] = useState(false);
  const pendingRef = useRef<PasswordRequest | null>(null);
  const { enabled: biometricEnabled } = useBiometric();
  const { wallet } = useWallet();

  // Biometric caching for standalone gate (Case 2)
  const lastBiometricPromptRef = useRef<number | null>(null);

  const requestPassword = useCallback(
    async (hasPassword: boolean, targetFingerprint?: number): Promise<string | null | undefined> => {
      const fingerprint = targetFingerprint ?? wallet?.fingerprint;

      // Case 1: No password, no biometric → no auth needed
      if (!hasPassword && !biometricEnabled) {
        return null;
      }

      // Case 2: No password, biometric enabled → standalone biometric gate with 5-min cache
      if (!hasPassword && biometricEnabled && isMobile) {
        const now = performance.now();
        if (
          lastBiometricPromptRef.current !== null &&
          now - lastBiometricPromptRef.current < BIOMETRIC_CACHE_MS
        ) {
          return null; // Within cache window, skip prompt
        }

        try {
          const { authenticate } = await import('@tauri-apps/plugin-biometric');
          await authenticate('Authenticate to continue', {
            allowDeviceCredential: false,
          });
          lastBiometricPromptRef.current = now;
          return null;
        } catch {
          return undefined; // biometric failed/cancelled
        }
      }

      // Case 3: Has password, biometric enabled → try keychain first
      // If keychain returns a stale password, the backend will reject it and the
      // "Incorrect password" toast fires. The user can cancel the biometric prompt
      // on the next attempt to fall through to the password dialog.
      if (hasPassword && biometricEnabled && isMobile && fingerprint) {
        const stored = await keychainGet(keychainKey(fingerprint));
        if (stored !== null) {
          return stored;
        }
        // Fall through to password dialog if keychain retrieval fails
      }

      // Case 4: Has password → show dialog (fallback or no biometric)
      if (hasPassword) {
        return new Promise<string | null | undefined>((resolve) => {
          pendingRef.current = { resolve, fingerprint };
          setOpen(true);
        });
      }

      return null;
    },
    [biometricEnabled, wallet?.fingerprint],
  );

  const handleSubmit = useCallback(
    (password: string) => {
      setOpen(false);
      const fingerprint = pendingRef.current?.fingerprint;

      // Manual password entry: store in keychain for future biometric use
      if (fingerprint) {
        if (biometricEnabled && isMobile) {
          keychainSave(keychainKey(fingerprint), password);
        }
      }

      pendingRef.current?.resolve(password);
      pendingRef.current = null;
    },
    [biometricEnabled],
  );

  const handleCancel = useCallback(() => {
    setOpen(false);
    pendingRef.current?.resolve(undefined);
    pendingRef.current = null;
  }, []);

  const clearKeychainEntry = useCallback(async (fingerprint: number) => {
    await keychainRemove(keychainKey(fingerprint));
  }, []);

  const updateKeychainEntry = useCallback(
    async (fingerprint: number, newPassword: string) => {
      if (biometricEnabled && isMobile) {
        await keychainSave(keychainKey(fingerprint), newPassword);
      }
    },
    [biometricEnabled],
  );

  const clearAllKeychainEntries = useCallback(
    async (fingerprints: number[]) => {
      for (const fp of fingerprints) {
        await keychainRemove(keychainKey(fp));
      }
    },
    [],
  );

  return (
    <PasswordContext.Provider
      value={{
        requestPassword,
        clearKeychainEntry,
        updateKeychainEntry,
        clearAllKeychainEntries,
      }}
    >
      {children}
      <PasswordDialog
        open={open}
        onSubmit={handleSubmit}
        onCancel={handleCancel}
      />
    </PasswordContext.Provider>
  );
}

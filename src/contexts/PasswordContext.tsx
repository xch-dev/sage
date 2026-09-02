import { PasswordDialog } from '@/components/dialogs/PasswordDialog';
import { useBiometric } from '@/hooks/useBiometric';
import { platform } from '@tauri-apps/plugin-os';
import { createContext, ReactNode, useCallback, useRef, useState } from 'react';

const isMobile = platform() === 'ios' || platform() === 'android';

// Biometric caching interval (5 minutes)
const BIOMETRIC_CACHE_MS = 5 * 60 * 1000;

interface PasswordRequest {
  resolve: (password: string | null | undefined) => void;
}

export interface PasswordContextType {
  requestPassword: (hasPassword: boolean) => Promise<string | null | undefined>;
}

export const PasswordContext = createContext<PasswordContextType | undefined>(
  undefined,
);

export function PasswordProvider({ children }: { children: ReactNode }) {
  const [open, setOpen] = useState(false);
  const pendingRef = useRef<PasswordRequest | null>(null);
  const { enabled: biometricEnabled } = useBiometric();

  // Biometric caching for standalone gate
  const lastBiometricPromptRef = useRef<number | null>(null);

  const requestPassword = useCallback(
    async (hasPassword: boolean): Promise<string | null | undefined> => {
      // Case 1: Has password → password takes precedence, show dialog
      if (hasPassword) {
        return new Promise<string | null | undefined>((resolve) => {
          pendingRef.current = { resolve };
          setOpen(true);
        });
      }

      // Case 2: No password, biometric enabled → standalone biometric gate with 5-min cache
      if (biometricEnabled && isMobile) {
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

      // Case 3: No password, no biometric → no auth needed
      return null;
    },
    [biometricEnabled],
  );

  const handleSubmit = useCallback((password: string) => {
    setOpen(false);
    pendingRef.current?.resolve(password);
    pendingRef.current = null;
  }, []);

  const handleCancel = useCallback(() => {
    setOpen(false);
    pendingRef.current?.resolve(undefined);
    pendingRef.current = null;
  }, []);

  return (
    <PasswordContext.Provider value={{ requestPassword }}>
      {children}
      <PasswordDialog
        open={open}
        onSubmit={handleSubmit}
        onCancel={handleCancel}
      />
    </PasswordContext.Provider>
  );
}

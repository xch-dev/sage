import { PasswordDialog } from '@/components/dialogs/PasswordDialog';
import { createContext, ReactNode, useCallback, useRef, useState } from 'react';

interface PasswordRequest {
  resolve: (password: string | null) => void;
}

export interface PasswordContextType {
  /**
   * If the wallet has a password, shows the password dialog and returns the entered password.
   * If it doesn't, returns null immediately.
   * Returns null if the user cancels.
   */
  requestPassword: (hasPassword: boolean) => Promise<string | null>;
}

export const PasswordContext = createContext<PasswordContextType | undefined>(
  undefined,
);

export function PasswordProvider({ children }: { children: ReactNode }) {
  const [open, setOpen] = useState(false);
  const pendingRef = useRef<PasswordRequest | null>(null);

  const requestPassword = useCallback(
    (hasPassword: boolean): Promise<string | null> => {
      if (!hasPassword) {
        return Promise.resolve(null);
      }

      return new Promise((resolve) => {
        pendingRef.current = { resolve };
        setOpen(true);
      });
    },
    [],
  );

  const handleSubmit = useCallback((password: string) => {
    setOpen(false);
    pendingRef.current?.resolve(password);
    pendingRef.current = null;
  }, []);

  const handleCancel = useCallback(() => {
    setOpen(false);
    pendingRef.current?.resolve(null);
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

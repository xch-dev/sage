import { commands } from '@/bindings';
import { CustomError } from '@/contexts/ErrorContext';
import { logoutAndUpdateState } from '@/state';
import { useCallback, useEffect, useMemo, useState } from 'react';
import { useErrors } from './useErrors';

export default function useInitialization() {
  const { addError } = useErrors();

  const [initialized, setInitialized] = useState(false);

  // Memoize addError to prevent unnecessary re-renders
  const memoizedAddError = useMemo(() => addError, [addError]);

  const onInitialize = useCallback(async () => {
    try {
      await commands.initialize();
      setInitialized(true);

      // Add error handling for switchWallet
      try {
        await commands.switchWallet();
      } catch (switchError: unknown) {
        const switchCustomError = switchError as CustomError;
        console.error('Error switching wallet:', switchError);
        // Only add non-migration errors to be displayed
        if (switchCustomError.kind !== 'database_migration') {
          memoizedAddError(switchCustomError);
        }
      }
    } catch (error: unknown) {
      const customError = error as CustomError;

      // Check if this is a database migration, which is recoverable
      if (customError.kind === 'database_migration') {
        try {
          await logoutAndUpdateState();
        } catch (logoutError) {
          console.error('Error during logout:', logoutError);
          // If logout fails, we should still try to continue
          setInitialized(true);
        }
      } else {
        // Only add non-migration errors to be displayed
        memoizedAddError(customError);
        console.error('Unrecoverable initialization error', error);
      }
    }
  }, [memoizedAddError]);

  useEffect(() => {
    if (!initialized) {
      onInitialize();
    }
  }, [initialized, onInitialize]);

  return initialized;
}

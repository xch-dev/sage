import { commands } from '@/bindings';
import { CustomError } from '@/contexts/ErrorContext';
import { logoutAndUpdateState } from '@/state';
import { useCallback, useEffect, useMemo, useState } from 'react';
import { useErrors } from './useErrors';

// Shared function to handle initialization errors
const handleInitializationError = async (
  error: unknown,
  addError: (error: CustomError) => void,
  setInitialized?: (value: boolean) => void,
) => {
  console.error('Error during initialization:', error);
  const customError = error as CustomError;

  // Check if this is a database migration, which is recoverable
  if (customError.kind === 'database_migration') {
    try {
      await logoutAndUpdateState();
    } catch (logoutError) {
      console.error('Error during logout:', logoutError);
      // If logout fails, we should still try to continue
      if (setInitialized) {
        setInitialized(true);
      }
    }
  } else {
    // Only add non-migration errors to be displayed
    addError(customError);
    console.error('Unrecoverable initialization error', error);
  }
};

export default function useInitialization() {
  const { addError } = useErrors();

  const [initialized, setInitialized] = useState(false);

  // Memoize addError to prevent unnecessary re-renders
  const memoizedAddError = useMemo(() => addError, [addError]);

  const onInitialize = useCallback(async () => {
    try {
      await commands.initialize();
      setInitialized(true);
      await commands.switchWallet();
    } catch (error: unknown) {
      await handleInitializationError(error, memoizedAddError, setInitialized);
    }
  }, [memoizedAddError]);

  useEffect(() => {
    if (!initialized) {
      onInitialize();
    }
  }, [initialized, onInitialize]);

  return initialized;
}

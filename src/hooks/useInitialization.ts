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
      console.log('Logging out and updating state');
      await logoutAndUpdateState();
    } catch (logoutError) {
      console.error('Error during logout:', logoutError);
      // If logout fails, we should still try to continue

    }
  } else {
    // Only add non-migration errors to be displayed
    addError(customError);
    console.error('Unrecoverable initialization error', error);
  }
  console.log('leaving handleInitializationError');
};

export default function useInitialization() {
  const { addError } = useErrors();

  const [initialized, setInitialized] = useState(false);

  // Memoize addError to prevent unnecessary re-renders
  const memoizedAddError = useMemo(() => addError, [addError]);

  const onInitialize = useCallback(async () => {
    try {
      console.log('initializing');
      await commands.initialize();
      console.log('initialized');

      await commands.switchWallet();
      console.log('switched wallet');
    } catch (error: unknown) {
      console.log('error in onInitialize', error);
      await handleInitializationError(error, memoizedAddError, setInitialized);
    }
    finally {
      console.log('onInitialize finally');
      setInitialized(true);
      console.log('set initialized to true');
    }
    console.log('leaving onInitialize');
  }, [memoizedAddError]);

  useEffect(() => {
    if (!initialized) {
      onInitialize().then(() => {
        console.log('onInitialize finished');
      });
    }
  }, [initialized, onInitialize]);

  return initialized;
}

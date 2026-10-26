import { commands } from '@/bindings';
import { CustomError } from '@/contexts/ErrorContext';
import { toCustomError } from '@/lib/errors';
import { logoutAndUpdateState } from '@/state';
import { useCallback, useEffect, useState } from 'react';
import { useErrors } from './useErrors';

// Shared function to handle initialization errors
const handleInitializationError = async (
  error: unknown,
  addError: (error: CustomError) => void,
) => {
  console.error('Error during initialization:', error);
  const customError = toCustomError(error);

  // Check if this is a database migration, which is recoverable
  if (customError.kind === 'database_migration') {
    try {
      await logoutAndUpdateState();
    } catch (logoutError) {
      console.error('Error during logout:', logoutError);
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
  const [startupError, setStartupError] = useState<CustomError | null>(null);

  const onInitialize = useCallback(async () => {
    try {
      await commands.initialize();
    } catch (error: unknown) {
      // The backend can't be used safely if it failed to initialize, so this blocks the app
      console.error('Startup failed:', error);
      setStartupError(toCustomError(error));
      setInitialized(true);
      return;
    }

    try {
      await commands.switchWallet();
    } catch (error: unknown) {
      await handleInitializationError(error, addError);
    } finally {
      setInitialized(true);
    }
  }, [addError]);

  useEffect(() => {
    if (!initialized) {
      onInitialize();
    }
  }, [initialized, onInitialize]);

  return { initialized, startupError };
}

import { commands } from '@/bindings';
import { CustomError } from '@/contexts/ErrorContext';
import { logoutAndUpdateState } from '@/state';
import { useCallback, useEffect, useState } from 'react';
import { useErrors } from './useErrors';

export default function useInitialization() {
  const { addError } = useErrors();

  const [initialized, setInitialized] = useState(false);

  const onInitialize = useCallback(async () => {
    try {
      await commands.initialize();
      setInitialized(true);
      await commands.switchWallet();
    } catch (error: unknown) {
      const customError = error as CustomError;

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
    }
  }, [addError]);

  useEffect(() => {
    if (!initialized) {
      onInitialize();
    }
  }, [initialized, onInitialize]);

  return initialized;
}

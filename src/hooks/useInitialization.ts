import { commands } from '@/bindings';
import { useCallback, useEffect, useState } from 'react';
import { useErrors } from './useErrors';
import { logoutAndUpdateState } from '@/state';

export default function useInitialization() {
  const { addError } = useErrors();

  const [initialized, setInitialized] = useState(false);

  const onInitialize = useCallback(async () => {
    commands
      .initialize()
      .then(() => setInitialized(true))
      .catch(async (error) => {
        // Always add the error to be displayed
        addError(error);

        // Check if this is a database migration, which is recoverable
        if (error.kind === 'database_migration') {
          try {
            await logoutAndUpdateState();
            console.log('Logged out due to database migration error');
            // Now mark as initialized so the app can proceed
            setInitialized(true);
          } catch (logoutError) {
            console.error('Error during logout:', logoutError);
          }
        } else {
          console.error('Unrecoverable initialization error', error);
        }
      });
  }, [addError]);

  useEffect(() => {
    if (!initialized) {
      onInitialize();
    }
  }, [initialized, onInitialize]);

  return initialized;
}

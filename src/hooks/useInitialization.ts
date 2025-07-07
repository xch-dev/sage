import { commands } from '@/bindings';
import { useCallback, useEffect, useState } from 'react';
import { useErrors } from './useErrors';
import { logoutAndUpdateState } from '@/state';

export default function useInitialization() {
  const { addError } = useErrors();

  const [initialized, setInitialized] = useState(false);

  const onInitialize = useCallback(async () => {
    try {
      await commands.initialize();
      setInitialized(true);
      await commands.switchWallet();
    } catch (error: any) {
      console.warn('onInitialize');

      // Check if this is a database migration, which is recoverable
      if (error.kind === 'database_migration') {
        try {
          await logoutAndUpdateState();
        } catch (logoutError) {
          console.error('Error during logout:', logoutError);
        }
      } else {
        // Only add non-migration errors to be displayed
        addError(error);
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

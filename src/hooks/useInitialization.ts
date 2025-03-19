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
        // When there's an error (like DB migration issues), log out and show error
        addError(error);
        try {
          // Log out to ensure we go to login screen
          await logoutAndUpdateState();
        } catch (logoutError) {
          // Even if logout fails, we still want to proceed
          console.error('Error during logout:', logoutError);
        }
        // Mark as initialized so the app can proceed
        setInitialized(true);
      });
  }, [addError]);

  useEffect(() => {
    if (!initialized) {
      onInitialize();
    }
  }, [initialized, onInitialize]);

  return initialized;
}

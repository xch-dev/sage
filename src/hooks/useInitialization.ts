import { commands } from '@/bindings';
import { useCallback, useEffect, useState } from 'react';
import { useErrors } from './useErrors';

export default function useInitialization() {
  const { addError } = useErrors();

  const [initialized, setInitialized] = useState(false);

  const initialize = useCallback(async () => {
    commands
      .initialize()
      .then(() => setInitialized(true))
      .catch(addError);
  }, [addError]);

  useEffect(() => {
    if (!initialized) {
      initialize();
    }
  }, [initialized, initialize]);

  return initialized;
}

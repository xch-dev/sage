import { commands } from '@/bindings';
import { useCallback, useEffect, useState } from 'react';

export default function useInitialization() {
  const [initialized, setInitialized] = useState(false);

  const onInitialize = useCallback(async () => {
    try {
      commands.initialize().then((result) => {
        if (result.status === 'ok') {
          setInitialized(true);
        }
      });
    } catch (err: unknown) {
      console.error('Initialization failed', err);
      alert(err);
    }
  }, []);

  useEffect(() => {
    if (!initialized) {
      onInitialize();
    }
  }, [initialized, onInitialize]);

  return initialized;
}

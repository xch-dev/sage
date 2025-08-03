import { useErrors } from '@/hooks/useErrors';
import { commands, DidRecord, events } from '../bindings';
import { useEffect, useMemo, useState } from 'react';

export interface UseDidDataParams {
  launcherId: string;
}

export function useDidData({ launcherId }: UseDidDataParams) {
  const { addError } = useErrors();
  const [did, setDid] = useState<DidRecord | null>(null);
  const [isLoading, setIsLoading] = useState(false);

  const updateDid = useMemo(
    () => () => {
      if (!launcherId) return;

      setIsLoading(true);
      commands
        .getProfile({ launcher_id: launcherId })
        .then((data) => setDid(data.did))
        .catch(addError)
        .finally(() => setIsLoading(false));
    },
    [launcherId, addError],
  );

  useEffect(() => {
    updateDid();

    const unlisten = events.syncEvent.listen((event) => {
      const type = event.payload.type;
      if (
        type === 'coin_state' ||
        type === 'puzzle_batch_synced' ||
        type === 'did_info'
      ) {
        updateDid();
      }
    });

    return () => {
      unlisten.then((u) => u());
    };
  }, [updateDid]);

  return {
    did,
    isLoading,
    updateDid,
  };
}

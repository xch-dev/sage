import { commands, DidRecord, events } from '@/bindings';
import { useCallback, useEffect, useState } from 'react';
import { useErrors } from './useErrors';

export function useDids() {
  const { addError } = useErrors();

  const [dids, setDids] = useState<DidRecord[]>([]);

  const updateDids = useCallback(async () => {
    return await commands
      .getDids({})
      .then((data) => setDids(data.dids))
      .catch(addError);
  }, [addError]);

  useEffect(() => {
    updateDids();

    const unlisten = events.syncEvent.listen((event) => {
      const type = event.payload.type;

      if (
        type === 'coin_state' ||
        type === 'puzzle_batch_synced' ||
        type === 'did_info'
      ) {
        updateDids();
      }
    });

    return () => {
      unlisten.then((u) => u());
    };
  }, [updateDids]);

  return {
    dids: dids.sort((a, b) => {
      if (a.visible !== b.visible) {
        return a.visible ? -1 : 1;
      }

      if (a.name && b.name) {
        return a.name.localeCompare(b.name);
      } else if (a.name) {
        return -1;
      } else if (b.name) {
        return 1;
      } else {
        return a.coin_id.localeCompare(b.coin_id);
      }
    }),
    updateDids,
  };
}

import { commands, DidRecord, events } from '@/bindings';
import { useEffect, useState } from 'react';

export function useDids() {
  const [dids, setDids] = useState<DidRecord[]>([]);

  const updateDids = async () => {
    return await commands.getDids().then((result) => {
      if (result.status === 'ok') {
        setDids(result.data);
      } else {
        throw new Error('Failed to get DIDs');
      }
    });
  };

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
  }, []);

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

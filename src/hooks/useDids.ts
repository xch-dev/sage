import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { commands, DidRecord, events } from '../bindings';
import { CustomError } from '../contexts/ErrorContext';
import { useWallet } from '../contexts/WalletContext';
import { useErrors } from './useErrors';

export function useDids() {
  const { addError } = useErrors();
  const { wallet } = useWallet();
  const isMountedRef = useRef(false);
  const [dids, setDids] = useState<DidRecord[]>([]);
  const [loading, setLoading] = useState(false);

  const updateDids = useCallback(async () => {
    // Only fetch DIDs if there's an authenticated wallet
    if (!wallet) {
      setDids([]);
      return;
    }

    setLoading(true);
    try {
      const data = await commands.getDids({});
      if (isMountedRef.current) {
        setDids(data.dids);
      }
    } catch (error) {
      if (isMountedRef.current) {
        addError(error as CustomError);
      }
    } finally {
      if (isMountedRef.current) {
        setLoading(false);
      }
    }
  }, [addError, wallet]);

  useEffect(() => {
    isMountedRef.current = true;

    updateDids();

    const unlisten = events.syncEvent.listen((event) => {
      const type = event.payload.type;

      if (
        type === 'coin_state' ||
        type === 'puzzle_batch_synced' ||
        type === 'did_info'
      ) {
        if (isMountedRef.current) {
          updateDids();
        }
      }
    });

    return () => {
      isMountedRef.current = false;
      unlisten.then((u) => u());
    };
  }, [updateDids]);

  // Sort DIDs with visible first, then by name, then by coin_id
  const sortedDids = useMemo(() => {
    return [...dids].sort((a, b) => {
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
    });
  }, [dids]);

  return {
    dids: sortedDids,
    updateDids,
    loading,
  };
}

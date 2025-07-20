import {
  createContext,
  ReactNode,
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
} from 'react';
import { commands, DidRecord, events } from '../bindings';
import { useErrors } from '../hooks/useErrors';
import { CustomError } from './ErrorContext';

export interface DidContextType {
  dids: DidRecord[];
  updateDids: () => Promise<void>;
}

export const DidContext = createContext<DidContextType | undefined>(undefined);

export function DidProvider({ children }: { children: ReactNode }) {
  const { addError } = useErrors();
  const isMountedRef = useRef(false);
  const [dids, setDids] = useState<DidRecord[]>([]);

  const updateDids = useCallback(async () => {
    try {
      const data = await commands.getDids({});
      if (isMountedRef.current) {
        setDids(data.dids);
      }
    } catch (error) {
      if (isMountedRef.current) {
        addError(error as CustomError);
      }
    }
  }, [addError]);

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
  }, [addError, updateDids]);

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

  return (
    <DidContext.Provider value={{ dids: sortedDids, updateDids }}>
      {children}
    </DidContext.Provider>
  );
}

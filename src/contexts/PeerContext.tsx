import { createContext, ReactNode, useEffect, useRef, useState } from 'react';
import { commands, PeerRecord } from '../bindings';
import { useErrors } from '../hooks/useErrors';
import { CustomError } from './ErrorContext';

export interface PeerContextType {
  peers: PeerRecord[] | null;
}

export const PeerContext = createContext<PeerContextType | undefined>(
  undefined,
);

export function PeerProvider({ children }: { children: ReactNode }) {
  const { addError } = useErrors();
  const [peers, setPeers] = useState<PeerRecord[] | null>(null);
  const isMountedRef = useRef(false);

  useEffect(() => {
    isMountedRef.current = true;
    const abortController = new AbortController();

    const updatePeers = async () => {
      try {
        const data = await commands.getPeers({});
        if (isMountedRef.current && !abortController.signal.aborted) {
          setPeers(data.peers);
        }
      } catch (error) {
        const customError = error as CustomError;
        // Don't add unauthorized errors - they're expected during wallet transitions
        if (
          isMountedRef.current &&
          !abortController.signal.aborted &&
          customError.kind !== 'unauthorized'
        ) {
          addError(customError);
        }
      }
    };

    updatePeers();
    const interval = setInterval(() => {
      if (!abortController.signal.aborted) {
        updatePeers();
      }
    }, 5000);

    return () => {
      isMountedRef.current = false;
      abortController.abort();
      clearInterval(interval);
    };
  }, [addError]);

  return (
    <PeerContext.Provider value={{ peers }}>{children}</PeerContext.Provider>
  );
}

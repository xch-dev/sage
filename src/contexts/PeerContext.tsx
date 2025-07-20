import { createContext, ReactNode, useEffect, useRef, useState } from 'react';
import { commands, PeerRecord } from '../bindings';
import { useErrors } from '../hooks/useErrors';

export interface PeerContextType {
  peers: PeerRecord[] | null;
}

export const PeerContext = createContext<PeerContextType | undefined>(
  undefined,
);

export function PeerProvider({ children }: { children: ReactNode }) {
  const { addError } = useErrors();
  const [peers, setPeers] = useState<PeerRecord[] | null>(null);
  const isMountedRef = useRef(true);

  useEffect(() => {
    isMountedRef.current = true;

    const updatePeers = () => {
      commands
        .getPeers({})
        .then((data) => {
          if (isMountedRef.current) {
            setPeers(data.peers);
          }
        })
        .catch((error) => {
          if (isMountedRef.current) {
            addError(error);
          }
        });
    };

    updatePeers();
    const interval = setInterval(updatePeers, 5000);

    return () => {
      isMountedRef.current = false;
      clearInterval(interval);
    };
  }, [addError]);

  return (
    <PeerContext.Provider value={{ peers }}>{children}</PeerContext.Provider>
  );
}

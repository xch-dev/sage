import { createContext, ReactNode, useEffect, useState } from 'react';
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

  useEffect(() => {
    const updatePeers = () => {
      commands
        .getPeers({})
        .then((data) => setPeers(data.peers))
        .catch(addError);
    };

    updatePeers();
    const interval = setInterval(updatePeers, 5000);
    return () => clearInterval(interval);
  }, [addError]);

  return (
    <PeerContext.Provider value={{ peers }}>{children}</PeerContext.Provider>
  );
}

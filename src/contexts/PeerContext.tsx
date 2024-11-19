import {
  createContext,
  ReactNode,
  useContext,
  useEffect,
  useState,
} from 'react';
import { commands, PeerRecord } from '../bindings';

interface PeerContextType {
  peers: PeerRecord[] | null;
}

const PeerContext = createContext<PeerContextType | undefined>(undefined);

export function PeerProvider({ children }: { children: ReactNode }) {
  const [peers, setPeers] = useState<PeerRecord[] | null>(null);

  useEffect(() => {
    const updatePeers = () => {
      commands.getPeers({}).then((res) => {
        if (res.status === 'ok') {
          setPeers(res.data.peers);
        }
      });
    };

    updatePeers();
    const interval = setInterval(updatePeers, 5000);

    return () => clearInterval(interval);
  }, []);

  return (
    <PeerContext.Provider value={{ peers }}>{children}</PeerContext.Provider>
  );
}

export function usePeers() {
  const context = useContext(PeerContext);
  if (context === undefined) {
    throw new Error('usePeers must be used within a PeerProvider');
  }
  return context;
}

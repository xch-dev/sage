import { useContext } from 'react';
import { PeerContext } from '../contexts/PeerContext';

export function usePeers() {
  const context = useContext(PeerContext);
  if (context === undefined) {
    throw new Error('usePeers must be used within a PeerProvider');
  }
  return context;
}

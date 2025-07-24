import { useContext } from 'react';
import { DidContext } from '../contexts/DidContext';

export function useDids() {
  const context = useContext(DidContext);
  if (context === undefined) {
    throw new Error('useDids must be used within a DidProvider');
  }
  return context;
}

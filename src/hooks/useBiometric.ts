import { BiometricContext } from '@/contexts/BiometricContext';
import { useContext } from 'react';

export function useBiometric() {
  const context = useContext(BiometricContext);
  if (!context) {
    throw new Error('useBiometric must be used within a BiometricProvider');
  }
  return context;
}

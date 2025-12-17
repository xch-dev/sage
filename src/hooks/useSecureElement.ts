import { SecureElementContext } from '@/contexts/SecureElementContext';
import { useContext } from 'react';

export function useSecureElement() {
  const context = useContext(SecureElementContext);
  if (!context) {
    throw new Error(
      'useSecureElement must be used within a SecureElementProvider',
    );
  }
  return context;
}

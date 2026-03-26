import { PasswordContext } from '@/contexts/PasswordContext';
import { useContext } from 'react';

export function usePassword() {
  const context = useContext(PasswordContext);
  if (!context) {
    throw new Error('usePassword must be used within a PasswordProvider');
  }
  return context;
}

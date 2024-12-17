import { useContext } from 'react';
import { ErrorContext } from '../contexts/ErrorContext';

export function useErrors() {
  const context = useContext(ErrorContext);
  if (context === undefined) {
    throw new Error('useErrors must be used within a ErrorProvider');
  }
  return context;
}

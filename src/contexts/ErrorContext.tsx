import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { createContext, ReactNode, useCallback, useState } from 'react';
import { ErrorKind } from '../bindings';

export interface CustomError {
  kind: ErrorKind | 'walletconnect' | 'upload' | 'invalid' | 'dexie';
  reason: string;
}

export interface ErrorContextType {
  errors: CustomError[];
  addError: (error: CustomError | unknown) => void;
}

export function toCustomError(error: unknown): CustomError {
  // If it's already a CustomError, return as-is
  if (
    error &&
    typeof error === 'object' &&
    'kind' in error &&
    'reason' in error
  ) {
    return error as CustomError;
  }

  // If it's a standard Error object
  if (error instanceof Error) {
    return {
      kind: 'internal',
      reason: error.message || 'An error occurred',
    };
  }

  // If it's a string
  if (typeof error === 'string') {
    return {
      kind: 'internal',
      reason: error,
    };
  }

  // Check for objects with a message property (common in many error types)
  if (error && typeof error === 'object' && 'message' in error) {
    return {
      kind: 'internal',
      reason: String((error as { message: unknown }).message),
    };
  }

  // Fallback for unknown error types
  return {
    kind: 'internal',
    reason: 'An unknown error occurred',
  };
}

export const ErrorContext = createContext<ErrorContextType | undefined>(
  undefined,
);

export function ErrorProvider({ children }: { children: ReactNode }) {
  const [errors, setErrors] = useState<CustomError[]>([]);

  const addError = useCallback((error: CustomError | unknown) => {
    const customError = toCustomError(error);
    setErrors((prevErrors) => [...prevErrors, customError]);
  }, []);

  return (
    <ErrorContext.Provider value={{ errors, addError }}>
      {children}

      {errors.length > 0 && (
        <ErrorDialog
          error={errors[0]}
          setError={() => setErrors((prevErrors) => prevErrors.slice(1))}
        />
      )}
    </ErrorContext.Provider>
  );
}

export interface ErrorDialogProps {
  error: CustomError | null;
  setError: (error: CustomError | null) => void;
}

export default function ErrorDialog({ error, setError }: ErrorDialogProps) {
  let kind: string | null;

  switch (error?.kind) {
    case 'api':
      kind = 'API';
      break;

    case 'internal':
      kind = 'Internal';
      break;

    case 'not_found':
      kind = 'Not Found';
      break;

    case 'unauthorized':
      kind = 'Auth';
      break;

    case 'wallet':
      kind = 'Wallet';
      break;

    case 'walletconnect':
      kind = 'WalletConnect';
      break;

    case 'upload':
      kind = 'Upload';
      break;

    case 'nfc':
      kind = 'NFC';
      break;

    case 'database_migration':
      kind = 'Database Migration';
      break;

    case 'dexie':
      kind = 'Dexie';
      break;

    default:
      kind = null;
  }

  return (
    <Dialog open={error !== null} onOpenChange={() => setError(null)}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{kind ? `${kind} ` : ''}Error</DialogTitle>
          <DialogDescription className='break-words hyphens-auto'>
            {error?.reason}
          </DialogDescription>
        </DialogHeader>
        <DialogFooter>
          <Button onClick={() => setError(null)} autoFocus>
            Ok
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

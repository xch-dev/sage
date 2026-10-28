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
import { ErrorKind, RecipientError } from '../bindings';

export interface CustomError {
  kind: ErrorKind | 'walletconnect' | 'upload' | 'invalid' | 'cancelled';
  reason: string;
  recipients?: RecipientError[] | null;
}

export interface ErrorContextType {
  errors: CustomError[];
  addError: (error: CustomError) => void;
}

export const ErrorContext = createContext<ErrorContextType | undefined>(
  undefined,
);

export function ErrorProvider({ children }: { children: ReactNode }) {
  const [errors, setErrors] = useState<CustomError[]>([]);

  const addError = useCallback((error: CustomError) => {
    // Skip unauthorized errors - they're expected during wallet transitions
    // and redundant when not logged in (user already knows they need to log in)
    if (error.kind === 'unauthorized') {
      return;
    }
    // The user chose not to continue, so there's nothing to report.
    if (error.kind === 'cancelled') {
      return;
    }
    setErrors((prevErrors) => [...prevErrors, error]);
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

    case 'recipient':
      kind = 'Recipient';
      break;

    case 'already_running':
      kind = 'Already Running';
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
            {error?.recipients?.length ? null : error?.reason}
          </DialogDescription>
        </DialogHeader>
        {error?.recipients?.length ? (
          <div className='space-y-2 max-h-[50vh] overflow-auto text-sm'>
            {error.recipients.map((recipient) => (
              <div
                key={recipient.recipient}
                className='rounded-md border p-2 space-y-1'
              >
                <div className='font-medium break-all'>
                  {recipient.recipient}
                </div>
                <div className='text-muted-foreground break-words'>
                  {recipient.reason}
                </div>
              </div>
            ))}
          </div>
        ) : null}
        <DialogFooter>
          <Button onClick={() => setError(null)} autoFocus>
            Ok
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

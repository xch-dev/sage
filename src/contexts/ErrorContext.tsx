import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import {
  createContext,
  ReactNode,
  useEffect,
  useMemo,
  useRef,
  useState,
} from 'react';
import { type Error } from '../bindings';

export interface ErrorContextType {
  errors: Error[];
  addError: (error: Error) => void;
}

export const ErrorContext = createContext<ErrorContextType | undefined>(
  undefined,
);

export function ErrorProvider({ children }: { children: ReactNode }) {
  const [errors, setErrors] = useState<Error[]>([]);
  const errorsRef = useRef(errors);

  useEffect(() => {
    errorsRef.current = errors;
  }, [errors]);

  const addError = useMemo(
    () => (error: Error) => setErrors([...errorsRef.current, error]),
    [],
  );

  return (
    <ErrorContext.Provider value={{ errors, addError }}>
      {children}

      {errors.length > 0 && (
        <ErrorDialog
          error={errors[0]}
          setError={() => setErrors(errors.slice(1))}
        />
      )}
    </ErrorContext.Provider>
  );
}

export interface ErrorDialogProps {
  error: Error | null;
  setError: (error: Error | null) => void;
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

    default:
      kind = null;
  }

  return (
    <Dialog open={error !== null} onOpenChange={() => setError(null)}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{kind ? `${kind} ` : ''}Error</DialogTitle>
          <DialogDescription>{error?.reason}</DialogDescription>
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

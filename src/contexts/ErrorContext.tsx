import {
  Dialog,
  DialogContent,
  DialogDescription,
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

      <Dialog
        open={errors.length > 0}
        onOpenChange={(open) => {
          if (!open) {
            setErrors([]);
          }
        }}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Errors</DialogTitle>
            <DialogDescription>
              There are {errors.length} errors.
            </DialogDescription>
          </DialogHeader>
        </DialogContent>
      </Dialog>
    </ErrorContext.Provider>
  );
}

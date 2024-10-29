import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { LoaderCircleIcon } from 'lucide-react';
import { useState } from 'react';
import { TransactionSummary } from '../bindings';

export interface ConfirmationDialogProps {
  summary: TransactionSummary | null;
  close: () => void;
  confirm: () => Promise<void>;
}

export default function ConfirmationDialog({
  summary,
  close,
  confirm,
}: ConfirmationDialogProps) {
  const [pending, setPending] = useState(false);

  return (
    <Dialog
      open={!!summary}
      onOpenChange={() => {
        close();
        setPending(false);
      }}
    >
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Confirm transaction?</DialogTitle>
          <DialogDescription>The fee is {summary?.fee}</DialogDescription>
        </DialogHeader>
        <DialogFooter>
          <Button
            onClick={() => {
              close();
              setPending(false);
            }}
          >
            Cancel
          </Button>
          <Button
            onClick={() => {
              setPending(true);
              confirm().then(() => {
                close();
                setPending(false);
              });
            }}
            disabled={pending}
          >
            {pending && (
              <LoaderCircleIcon className='mr-2 h-4 w-4 animate-spin' />
            )}
            {pending ? 'Submitting' : 'Submit'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

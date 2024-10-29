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
import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
} from './ui/accordion';

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
          <DialogDescription>
            {JSON.stringify(summary)}

            <Accordion type='single' collapsible>
              <AccordionItem value='item-1'>
                <AccordionTrigger>Is it accessible?</AccordionTrigger>
                <AccordionContent>
                  Yes. It adheres to the WAI-ARIA design pattern.
                </AccordionContent>
              </AccordionItem>
            </Accordion>
          </DialogDescription>
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

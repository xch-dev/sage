import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { useWalletState } from '@/state';
import { LoaderCircleIcon, MoveRight } from 'lucide-react';
import { useState } from 'react';
import { Input, Output, TransactionSummary } from '../bindings';
import { Badge } from './ui/badge';

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
            {summary?.inputs.map((input) => {
              return <InputItem key={input.coin_id} input={input} />;
            })}
          </DialogDescription>
        </DialogHeader>
        <DialogFooter>
          <Button
            variant='ghost'
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

interface InputProps {
  input: Input;
}

function InputItem({ input }: InputProps) {
  const walletState = useWalletState();

  return (
    <div className='flex flex-col gap-1 mt-2 w-full font-medium text-left text-neutral-900 dark:text-neutral-200 bg-neutral-200 dark:bg-neutral-900 p-3 rounded-lg'>
      <div className='flex items-center gap-2'>
        <Badge>{walletState.sync.unit.ticker}</Badge>
        <span>
          {input.amount} {walletState.sync.unit.ticker}
        </span>
      </div>

      {input.outputs.map((output) => (
        <OutputItem key={output.coin_id} output={output} />
      ))}
    </div>
  );
}

interface OutputProps {
  output: Output;
}

function OutputItem({ output }: OutputProps) {
  const walletState = useWalletState();

  return (
    <div className='flex items-start gap-2 ml-4 truncate'>
      <MoveRight />
      <div>
        <div className='truncate w-[250px]'>{output.address}</div>
        <div>
          {output.amount} {walletState.sync.unit.ticker}
        </div>
      </div>
    </div>
  );
}

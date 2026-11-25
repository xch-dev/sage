import { commands } from '@/bindings';
import { CustomError } from '@/contexts/ErrorContext';
import { useErrors } from '@/hooks/useErrors';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { useId, useState } from 'react';
import { Button } from '../ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '../ui/dialog';
import { Label } from '../ui/label';
import { LoadingButton } from '../ui/loading-button';
import { Textarea } from '../ui/textarea';

export interface AddKeyDialogProps {
  open: boolean;
  setOpen: (open: boolean) => void;
  onAdded: (fingerprint: number) => void;
}

export function AddKeyDialog({ open, setOpen, onAdded }: AddKeyDialogProps) {
  const { addError } = useErrors();
  const keyId = useId();

  const [key, setKey] = useState('');
  const [pending, setPending] = useState(false);

  const close = () => {
    setOpen(false);
    setKey('');
  };

  const submit = async () => {
    setPending(true);

    try {
      const { fingerprint } = await commands.addKey({
        key: key.trim(),
      });
      onAdded(fingerprint);
      close();
    } catch (error) {
      addError(error as CustomError);
    } finally {
      setPending(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={(open) => !open && close()}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>
            <Trans>Add Key</Trans>
          </DialogTitle>
          <DialogDescription>
            <Trans>
              Add a key to the keychain without creating a wallet for it. A
              mnemonic or secret key is stored along with its public key. To use
              a key as a wallet, import it as a wallet instead.
            </Trans>
          </DialogDescription>
        </DialogHeader>

        <div className='flex flex-col gap-4'>
          <div className='flex flex-col gap-2'>
            <Label htmlFor={keyId}>
              <Trans>Mnemonic or key</Trans>
            </Label>
            <Textarea
              id={keyId}
              className='resize-none h-24'
              autoCorrect='off'
              autoCapitalize='off'
              autoComplete='off'
              spellCheck={false}
              placeholder={t`Enter a mnemonic, secret key or public key`}
              value={key}
              onChange={(event) => setKey(event.target.value)}
            />
          </div>
        </div>

        <DialogFooter>
          <Button variant='outline' onClick={close}>
            <Trans>Cancel</Trans>
          </Button>
          <LoadingButton
            loading={pending}
            disabled={!key.trim()}
            onClick={submit}
          >
            <Trans>Add</Trans>
          </LoadingButton>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

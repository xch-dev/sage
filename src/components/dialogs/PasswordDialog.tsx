import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { KeyRoundIcon } from 'lucide-react';
import { useCallback, useEffect, useRef, useState } from 'react';
import { Button } from '../ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '../ui/dialog';
import { Input } from '../ui/input';

export interface PasswordDialogProps {
  open: boolean;
  onSubmit: (password: string) => void;
  onCancel: () => void;
}

export function PasswordDialog({
  open,
  onSubmit,
  onCancel,
}: PasswordDialogProps) {
  const [password, setPassword] = useState('');
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (open) {
      setPassword('');
      setTimeout(() => inputRef.current?.focus(), 100);
    }
  }, [open]);

  const handleSubmit = useCallback(() => {
    onSubmit(password);
    setPassword('');
  }, [password, onSubmit]);

  const handleCancel = useCallback(() => {
    setPassword('');
    onCancel();
  }, [onCancel]);

  return (
    <Dialog open={open} onOpenChange={(isOpen) => !isOpen && handleCancel()}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle className='flex items-center gap-2'>
            <KeyRoundIcon className='h-5 w-5' />
            <Trans>Enter Password</Trans>
          </DialogTitle>
          <DialogDescription>
            <Trans>
              This wallet is password-protected. Enter your password to
              continue.
            </Trans>
          </DialogDescription>
        </DialogHeader>
        <Input
          ref={inputRef}
          type='password'
          placeholder={t`Password`}
          value={password}
          onChange={(e) => setPassword(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === 'Enter') {
              e.preventDefault();
              handleSubmit();
            }
          }}
        />
        <DialogFooter>
          <Button variant='outline' onClick={handleCancel}>
            <Trans>Cancel</Trans>
          </Button>
          <Button onClick={handleSubmit}>
            <Trans>Unlock</Trans>
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

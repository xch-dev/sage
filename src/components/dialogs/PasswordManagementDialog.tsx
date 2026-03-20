import { CustomError } from '@/contexts/ErrorContext';
import { commands } from '@/bindings';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { LoaderCircleIcon } from 'lucide-react';
import { useState } from 'react';
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
import { Label } from '../ui/label';

export type PasswordDialogMode = 'set' | 'change' | 'remove';

export interface PasswordManagementDialogProps {
  open: boolean;
  mode: PasswordDialogMode;
  fingerprint: number;
  onClose: () => void;
  onSuccess: () => void;
}

export function PasswordManagementDialog({
  open,
  mode,
  fingerprint,
  onClose,
  onSuccess,
}: PasswordManagementDialogProps) {
  const [oldPassword, setOldPassword] = useState('');
  const [newPassword, setNewPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const resetState = () => {
    setOldPassword('');
    setNewPassword('');
    setConfirmPassword('');
    setError(null);
  };

  const handleClose = () => {
    resetState();
    onClose();
  };

  const handleSubmit = async () => {
    setError(null);

    if (mode !== 'remove' && newPassword !== confirmPassword) {
      setError(t`Passwords do not match`);
      return;
    }

    if (mode !== 'remove' && newPassword.length === 0) {
      setError(t`Password cannot be empty`);
      return;
    }

    setPending(true);
    try {
      await commands.changePassword({
        fingerprint,
        old_password: mode === 'set' ? '' : oldPassword,
        new_password: mode === 'remove' ? '' : newPassword,
      });

      resetState();
      onSuccess();
    } catch (err) {
      const customErr = err as CustomError;
      setError(customErr.reason || t`Incorrect password`);
    } finally {
      setPending(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={(isOpen) => !isOpen && handleClose()}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>
            {mode === 'set' && <Trans>Set Password</Trans>}
            {mode === 'change' && <Trans>Change Password</Trans>}
            {mode === 'remove' && <Trans>Remove Password</Trans>}
          </DialogTitle>
          <DialogDescription>
            {mode === 'set' && (
              <Trans>
                Set a password to protect transaction signing and secret key
                access. There is no way to recover a lost password.
              </Trans>
            )}
            {mode === 'change' && (
              <Trans>Enter your current password and choose a new one.</Trans>
            )}
            {mode === 'remove' && (
              <Trans>
                Enter your current password to remove password protection. Your
                wallet secrets will no longer require a password.
              </Trans>
            )}
          </DialogDescription>
        </DialogHeader>
        <div className='space-y-4'>
          {mode !== 'set' && (
            <div className='space-y-2'>
              <Label htmlFor='oldPassword'>
                <Trans>Current Password</Trans>
              </Label>
              <Input
                id='oldPassword'
                type='password'
                value={oldPassword}
                onChange={(e) => setOldPassword(e.target.value)}
                placeholder={t`Enter current password`}
                onKeyDown={(e) => {
                  if (e.key === 'Enter' && mode === 'remove') {
                    e.preventDefault();
                    handleSubmit();
                  }
                }}
              />
            </div>
          )}
          {mode !== 'remove' && (
            <>
              <div className='space-y-2'>
                <Label htmlFor='newPassword'>
                  {mode === 'set' ? (
                    <Trans>Password</Trans>
                  ) : (
                    <Trans>New Password</Trans>
                  )}
                </Label>
                <Input
                  id='newPassword'
                  type='password'
                  value={newPassword}
                  onChange={(e) => setNewPassword(e.target.value)}
                  placeholder={t`Enter password`}
                />
              </div>
              <div className='space-y-2'>
                <Label htmlFor='confirmPassword'>
                  <Trans>Confirm Password</Trans>
                </Label>
                <Input
                  id='confirmPassword'
                  type='password'
                  value={confirmPassword}
                  onChange={(e) => setConfirmPassword(e.target.value)}
                  placeholder={t`Confirm password`}
                  onKeyDown={(e) => {
                    if (e.key === 'Enter') {
                      e.preventDefault();
                      handleSubmit();
                    }
                  }}
                />
              </div>
            </>
          )}
          {error && <p className='text-sm text-destructive'>{error}</p>}
        </div>
        <DialogFooter className='gap-2'>
          <Button variant='outline' onClick={handleClose}>
            <Trans>Cancel</Trans>
          </Button>
          <Button
            onClick={handleSubmit}
            disabled={pending}
            variant={mode === 'remove' ? 'destructive' : 'default'}
          >
            {pending && (
              <LoaderCircleIcon className='mr-2 h-4 w-4 animate-spin' />
            )}
            {mode === 'set' && <Trans>Set Password</Trans>}
            {mode === 'change' && <Trans>Change Password</Trans>}
            {mode === 'remove' && <Trans>Remove Password</Trans>}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

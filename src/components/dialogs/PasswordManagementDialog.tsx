import { CustomError } from '@/contexts/ErrorContext';
import { commands } from '@/bindings';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { AlertTriangleIcon, LoaderCircleIcon } from 'lucide-react';
import { useState } from 'react';
import { Alert, AlertDescription, AlertTitle } from '../ui/alert';
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
import { Textarea } from '../ui/textarea';

export type PasswordDialogMode = 'set' | 'change' | 'remove';

export interface PasswordManagementDialogProps {
  open: boolean;
  mode: PasswordDialogMode;
  fingerprint: number;
  onClose: () => void;
  onSuccess: () => void;
}

function normalizeKey(key: string): string {
  return key
    .trim()
    .replace(/[^a-z]/gi, ' ')
    .split(/\s+/)
    .filter((item) => item.length > 0)
    .join(' ')
    .toLowerCase();
}

function normalizeHex(hex: string): string {
  let h = hex.trim().toLowerCase();
  if (h.startsWith('0x')) {
    h = h.slice(2);
  }
  return h;
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
  const [verificationKey, setVerificationKey] = useState('');
  const [pending, setPending] = useState(false);
  const [verifying, setVerifying] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const resetState = () => {
    setOldPassword('');
    setNewPassword('');
    setConfirmPassword('');
    setVerificationKey('');
    setError(null);
  };

  const handleClose = () => {
    resetState();
    onClose();
  };

  const verifyBackupKey = async (): Promise<boolean> => {
    setVerifying(true);
    try {
      const response = await commands.getSecretKey({
        fingerprint,
      });

      if (!response.secrets) {
        setError(t`Unable to retrieve wallet secrets for verification.`);
        return false;
      }

      const input = verificationKey.trim();
      const { mnemonic, secret_key } = response.secrets;

      // Check if input matches the mnemonic
      if (mnemonic && normalizeKey(input) === normalizeKey(mnemonic)) {
        return true;
      }

      // Check if input matches the secret key
      if (normalizeHex(input) === normalizeHex(secret_key)) {
        return true;
      }

      setError(
        t`The seed phrase or secret key you entered does not match this wallet. Please verify and try again.`,
      );
      return false;
    } catch (err) {
      const customErr = err as CustomError;
      setError(customErr.reason || t`Failed to verify key. Please try again.`);
      return false;
    } finally {
      setVerifying(false);
    }
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

    if (mode === 'set') {
      if (verificationKey.trim().length === 0) {
        setError(
          t`You must enter your seed phrase or secret key to verify you have a backup before setting a password.`,
        );
        return;
      }

      const verified = await verifyBackupKey();
      if (!verified) return;
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
                access.
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
          {mode === 'set' && (
            <Alert variant='destructive'>
              <AlertTriangleIcon className='h-4 w-4' />
              <AlertTitle>
                <Trans>No password recovery</Trans>
              </AlertTitle>
              <AlertDescription>
                <Trans>
                  If you lose or forget this password, your wallet and its keys
                  cannot be recovered without your seed phrase or secret key.
                  Make sure they are safely stored before continuing.
                </Trans>
              </AlertDescription>
            </Alert>
          )}

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
                    if (e.key === 'Enter' && mode !== 'set') {
                      e.preventDefault();
                      handleSubmit();
                    }
                  }}
                />
              </div>
            </>
          )}

          {mode === 'set' && (
            <div className='space-y-2'>
              <Label htmlFor='verificationKey'>
                <Trans>Verify Seed Phrase or Secret Key</Trans>
              </Label>
              <Textarea
                id='verificationKey'
                className='resize-none h-20'
                value={verificationKey}
                onChange={(e) => setVerificationKey(e.target.value)}
                placeholder={t`Enter your seed phrase or secret key to confirm you have a backup`}
                onKeyDown={(e) => {
                  if (e.key === 'Enter' && !e.shiftKey) {
                    e.preventDefault();
                    handleSubmit();
                  }
                }}
              />
              <p className='text-xs text-muted-foreground'>
                <Trans>
                  Enter your 12 or 24-word seed phrase or your secret key to
                  verify that you have a backup stored safely.
                </Trans>
              </p>
            </div>
          )}

          {error && <p className='text-sm text-destructive'>{error}</p>}
        </div>
        <DialogFooter className='gap-2'>
          <Button variant='outline' onClick={handleClose}>
            <Trans>Cancel</Trans>
          </Button>
          <Button
            onClick={handleSubmit}
            disabled={pending || verifying}
            variant={mode === 'remove' ? 'destructive' : 'default'}
          >
            {(pending || verifying) && (
              <LoaderCircleIcon className='mr-2 h-4 w-4 animate-spin' />
            )}
            {verifying ? (
              <Trans>Verifying...</Trans>
            ) : (
              <>
                {mode === 'set' && <Trans>Set Password</Trans>}
                {mode === 'change' && <Trans>Change Password</Trans>}
                {mode === 'remove' && <Trans>Remove Password</Trans>}
              </>
            )}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

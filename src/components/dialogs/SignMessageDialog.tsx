import { commands, SignMessageByAddressResponse } from '@/bindings';
import { AddressItem } from '@/components/AddressItem';
import { CustomError } from '@/contexts/ErrorContext';
import { useWallet } from '@/contexts/WalletContext';
import { useBiometric } from '@/hooks/useBiometric';
import { useErrors } from '@/hooks/useErrors';
import {
  fromHexStrict,
  isValidHexBytes,
  toHex,
  utf8ToBytes,
} from '@/lib/utils';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { LoaderCircleIcon } from 'lucide-react';
import { useEffect, useId, useState } from 'react';
import { ReadOnlyButton } from '../ReadOnlyButton';
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
import { Switch } from '../ui/switch';
import { Textarea } from '../ui/textarea';

export interface SignMessageDialogProps {
  open: boolean;
  setOpen: (open: boolean) => void;
  address?: string;
}

type MessageMode = 'text' | 'hex';

// The backend signs any all-hex string as raw bytes and anything else as UTF-8
// text, so always send hex. That way text which happens to look like hex (such
// as "cafe") is still signed as the characters the user typed.
function messageBytes(message: string, mode: MessageMode): Uint8Array | null {
  if (mode === 'text') return utf8ToBytes(message);
  if (!isValidHexBytes(message)) return null;

  const bytes = fromHexStrict(message);
  return bytes.length > 0 ? bytes : null;
}

export function SignMessageDialog({
  open,
  setOpen,
  address: initialAddress = '',
}: SignMessageDialogProps) {
  const { addError } = useErrors();
  const { isReadOnly } = useWallet();
  const { promptIfEnabled } = useBiometric();

  const addressId = useId();
  const messageId = useId();
  const hexModeId = useId();

  const [address, setAddress] = useState(initialAddress);
  const [message, setMessage] = useState('');
  const [mode, setMode] = useState<MessageMode>('text');
  const [addressError, setAddressError] = useState<string | null>(null);
  const [pending, setPending] = useState(false);
  const [result, setResult] = useState<SignMessageByAddressResponse | null>(
    null,
  );

  const bytes = message ? messageBytes(message, mode) : null;
  const hexError = mode === 'hex' && !!message && !bytes;

  useEffect(() => {
    if (open) {
      setAddress(initialAddress);
      setMessage('');
      setMode('text');
      setAddressError(null);
      setResult(null);
    }
  }, [open, initialAddress]);

  const sign = async () => {
    const trimmed = address.trim();

    if (!bytes) return;

    setPending(true);

    try {
      if (!(await commands.validateAddress(trimmed))) {
        setAddressError(t`Invalid address`);
        return;
      }

      if (!(await promptIfEnabled())) return;

      setResult(
        await commands.signMessageByAddress({
          address: trimmed,
          message: `0x${toHex(bytes)}`,
        }),
      );
    } catch (error) {
      addError(error as CustomError);
    } finally {
      setPending(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={(open) => !open && setOpen(false)}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>
            <Trans>Sign Message</Trans>
          </DialogTitle>
          <DialogDescription>
            <Trans>
              Prove that you own an address by signing a message with its key.
            </Trans>
          </DialogDescription>
        </DialogHeader>

        {result ? (
          <div className='flex flex-col gap-4 min-w-0'>
            <AddressItem label={t`Address`} address={address.trim()} />
            <AddressItem label={t`Public key`} address={result.publicKey} />
            <AddressItem label={t`Signature`} address={result.signature} />
          </div>
        ) : (
          <div className='flex flex-col gap-4'>
            <div className='flex flex-col gap-2'>
              <Label htmlFor={addressId}>
                <Trans>Address</Trans>
              </Label>
              <Input
                id={addressId}
                value={address}
                placeholder={t`Enter address`}
                aria-invalid={!!addressError}
                onChange={(e) => {
                  setAddress(e.target.value);
                  setAddressError(null);
                }}
              />
              {addressError && (
                <p className='text-sm text-destructive' role='alert'>
                  {addressError}
                </p>
              )}
            </div>
            <div className='flex flex-col gap-2'>
              <div className='flex items-center justify-between'>
                <Label htmlFor={messageId}>
                  <Trans>Message</Trans>
                </Label>
                <div className='flex items-center gap-2'>
                  <Label htmlFor={hexModeId}>
                    <Trans>Hex</Trans>
                  </Label>
                  <Switch
                    id={hexModeId}
                    checked={mode === 'hex'}
                    onCheckedChange={(checked) =>
                      setMode(checked ? 'hex' : 'text')
                    }
                  />
                </div>
              </div>
              <Textarea
                id={messageId}
                value={message}
                placeholder={
                  mode === 'hex' ? t`e.g. ff00ab` : t`Enter message to sign`
                }
                className='min-h-[100px] font-mono'
                aria-invalid={hexError}
                onChange={(e) => setMessage(e.target.value)}
              />
              {hexError ? (
                <p className='text-sm text-destructive' role='alert'>
                  <Trans>
                    Message must be valid hex bytes (even length, 0-9, a-f)
                  </Trans>
                </p>
              ) : (
                <p className='text-sm text-muted-foreground'>
                  {mode === 'hex' ? (
                    <Trans>Signed as raw hex bytes</Trans>
                  ) : (
                    <Trans>Signed as UTF-8 text</Trans>
                  )}
                </p>
              )}
            </div>
            {isReadOnly && (
              <p className='text-sm text-muted-foreground'>
                <Trans>
                  This is a read-only wallet, so it cannot sign messages.
                </Trans>
              </p>
            )}
          </div>
        )}

        <DialogFooter>
          {result ? (
            <>
              <Button variant='outline' onClick={() => setResult(null)}>
                <Trans>Sign Another</Trans>
              </Button>
              <Button onClick={() => setOpen(false)}>
                <Trans>Done</Trans>
              </Button>
            </>
          ) : (
            <>
              <Button variant='outline' onClick={() => setOpen(false)}>
                <Trans>Cancel</Trans>
              </Button>
              <ReadOnlyButton
                requiresSigning
                disabledTooltip={<Trans>Read-only wallets cannot sign</Trans>}
                disabled={pending || !address.trim() || !bytes}
                aria-busy={pending}
                onClick={sign}
              >
                {pending ? (
                  <>
                    <LoaderCircleIcon
                      className='mr-2 h-4 w-4 animate-spin'
                      aria-hidden='true'
                    />
                    <Trans>Signing</Trans>
                  </>
                ) : (
                  <Trans>Sign</Trans>
                )}
              </ReadOnlyButton>
            </>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

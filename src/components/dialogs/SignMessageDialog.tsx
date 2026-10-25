import { commands, SignMessageByAddressResponse } from '@/bindings';
import { AddressItem } from '@/components/AddressItem';
import { CustomError } from '@/contexts/ErrorContext';
import { useWallet } from '@/contexts/WalletContext';
import { useBiometric } from '@/hooks/useBiometric';
import { useErrors } from '@/hooks/useErrors';
import { t } from '@lingui/core/macro';
import { Plural, Trans } from '@lingui/react/macro';
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
import { Textarea } from '../ui/textarea';

export interface SignMessageDialogProps {
  open: boolean;
  setOpen: (open: boolean) => void;
  address?: string;
}

type MessageEncoding =
  | { kind: 'text' }
  | { kind: 'hex'; bytes: number }
  | { kind: 'oddHex' };

// Mirrors `parse_signature_message` in crates/sage/src/utils/parse.rs: a
// non-empty string of hex digits, with or without a 0x prefix, is decoded and
// signed as raw bytes. Anything else is signed as UTF-8 text.
function messageEncoding(message: string): MessageEncoding {
  const hex = message.startsWith('0x') ? message.slice(2) : message;

  if (!/^[0-9a-fA-F]+$/.test(hex)) return { kind: 'text' };
  if (hex.length % 2 !== 0) return { kind: 'oddHex' };

  return { kind: 'hex', bytes: hex.length / 2 };
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

  const [address, setAddress] = useState(initialAddress);
  const [message, setMessage] = useState('');
  const [addressError, setAddressError] = useState<string | null>(null);
  const [pending, setPending] = useState(false);
  const [result, setResult] = useState<SignMessageByAddressResponse | null>(
    null,
  );

  const encoding = messageEncoding(message);

  useEffect(() => {
    if (open) {
      setAddress(initialAddress);
      setMessage('');
      setAddressError(null);
      setResult(null);
    }
  }, [open, initialAddress]);

  const sign = async () => {
    const trimmed = address.trim();

    setPending(true);

    try {
      if (!(await commands.validateAddress(trimmed))) {
        setAddressError(t`Invalid address`);
        return;
      }

      if (!(await promptIfEnabled())) return;

      setResult(
        await commands.signMessageByAddress({ address: trimmed, message }),
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
              Messages made up only of hex digits, with or without a 0x prefix,
              are signed as raw bytes. Anything else is signed as text.
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
              <Label htmlFor={messageId}>
                <Trans>Message</Trans>
              </Label>
              <Textarea
                id={messageId}
                value={message}
                placeholder={t`Enter message to sign`}
                className='min-h-[100px] font-mono'
                aria-invalid={encoding.kind === 'oddHex'}
                onChange={(e) => setMessage(e.target.value)}
              />
              {encoding.kind === 'hex' && (
                <p className='text-sm text-muted-foreground'>
                  <Plural
                    value={encoding.bytes}
                    one='This will be signed as # byte of hex data, not as text.'
                    other='This will be signed as # bytes of hex data, not as text.'
                  />
                </p>
              )}
              {encoding.kind === 'oddHex' && (
                <p className='text-sm text-destructive' role='alert'>
                  <Trans>
                    Hex messages must have an even number of digits.
                  </Trans>
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
                disabled={
                  pending ||
                  !address.trim() ||
                  !message ||
                  encoding.kind === 'oddHex'
                }
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

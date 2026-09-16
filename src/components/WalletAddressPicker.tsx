import { commands, KeyInfo } from '@/bindings';
import { CustomError } from '@/contexts/ErrorContext';
import { useWallet } from '@/contexts/WalletContext';
import { useErrors } from '@/hooks/useErrors';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { WalletIcon } from 'lucide-react';
import { useEffect, useState } from 'react';
import { Button } from './ui/button';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from './ui/dropdown-menu';

interface WalletAddressPickerProps {
  onSelect: (address: string) => void;
}

// Tauri IPC rejects with a plain string (e.g. a missing-permission error)
// instead of the usual { kind, reason } shape when the command itself is
// unreachable, so callers here can't assume `e` is already a CustomError.
function toCustomError(e: unknown): CustomError {
  if (
    typeof e === 'object' &&
    e !== null &&
    'kind' in e &&
    'reason' in e &&
    typeof (e as CustomError).reason === 'string'
  ) {
    return e as CustomError;
  }

  return {
    kind: 'internal',
    reason: e instanceof Error ? e.message : String(e),
  };
}

export function WalletAddressPicker({ onSelect }: WalletAddressPickerProps) {
  const { wallet } = useWallet();
  const { addError } = useErrors();
  const [otherWallets, setOtherWallets] = useState<KeyInfo[]>([]);
  const [loadingFingerprint, setLoadingFingerprint] = useState<number | null>(
    null,
  );

  useEffect(() => {
    if (!wallet) return;

    commands
      .getKeys({})
      .then(({ keys }) => {
        setOtherWallets(
          keys.filter(
            (k) =>
              k.fingerprint !== wallet.fingerprint &&
              k.network_id === wallet.network_id,
          ),
        );
      })
      .catch((e) => addError(toCustomError(e)));
  }, [wallet, addError]);

  if (otherWallets.length === 0) return null;

  const handleSelect = async (fingerprint: number) => {
    if (!wallet) return;

    setLoadingFingerprint(fingerprint);
    try {
      const { address } = await commands.getWalletAddress({
        fingerprint,
        network_id: wallet.network_id,
      });
      onSelect(address);
    } catch (e) {
      addError(toCustomError(e));
    } finally {
      setLoadingFingerprint(null);
    }
  };

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button
          type='button'
          variant='ghost'
          size='sm'
          className='h-auto py-0.5 px-1.5 text-xs text-muted-foreground hover:text-foreground gap-1'
          aria-label={t`Insert address from another wallet`}
        >
          <WalletIcon className='h-3 w-3' aria-hidden='true' />
          <Trans>My wallets</Trans>
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align='start'>
        {otherWallets.map((w) => (
          <DropdownMenuItem
            key={w.fingerprint}
            disabled={loadingFingerprint === w.fingerprint}
            onSelect={() => handleSelect(w.fingerprint)}
          >
            {w.emoji && <span aria-hidden='true'>{w.emoji}</span>}
            <span>{w.name}</span>
          </DropdownMenuItem>
        ))}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}

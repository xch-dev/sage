import { commands, KeyInfo } from '@/bindings';
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
      .catch(addError);
  }, [wallet, addError]);

  if (otherWallets.length === 0) return null;

  const handleSelect = async (fingerprint: number) => {
    setLoadingFingerprint(fingerprint);
    try {
      const { address } = await commands.getWalletAddress({
        fingerprint,
        network_id: wallet!.network_id,
      });
      onSelect(address);
    } catch (e) {
      addError(e as Parameters<typeof addError>[0]);
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

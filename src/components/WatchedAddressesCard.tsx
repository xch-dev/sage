import { commands } from '@/bindings';
import { AddressItem } from '@/components/AddressItem';
import { Button } from '@/components/ui/button';
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card';
import { Textarea } from '@/components/ui/textarea';
import { CustomError } from '@/contexts/ErrorContext';
import { useWallet } from '@/contexts/WalletContext';
import { useErrors } from '@/hooks/useErrors';
import { splitAddresses, watchedAddresses } from '@/lib/walletKind';
import { fetchState } from '@/state';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { PlusIcon, TrashIcon } from 'lucide-react';
import { useState } from 'react';

/** Lists the addresses a watch wallet watches, and lets them be added or removed. */
export function WatchedAddressesCard() {
  const { wallet, setWallet } = useWallet();
  const { addError } = useErrors();
  const [input, setInput] = useState('');
  const [pending, setPending] = useState(false);

  const addresses = watchedAddresses(wallet) ?? [];
  const toAdd = splitAddresses(input);

  const update = async (change: () => Promise<unknown>) => {
    if (!wallet) return;

    setPending(true);

    try {
      await change();
      const data = await commands.getWallet({});
      setWallet(data.wallet);
      await fetchState();
      return true;
    } catch (error) {
      addError(error as CustomError);
      return false;
    } finally {
      setPending(false);
    }
  };

  const add = async () => {
    if (!wallet) return;

    const added = await update(() =>
      commands.addWatchAddresses({
        fingerprint: wallet.fingerprint,
        addresses: toAdd,
      }),
    );

    if (added) setInput('');
  };

  const remove = (address: string) => {
    if (!wallet) return;

    update(() =>
      commands.removeWatchAddresses({
        fingerprint: wallet.fingerprint,
        addresses: [address],
      }),
    );
  };

  return (
    <Card className='max-w-full'>
      <CardHeader>
        <CardTitle className='text-lg font-medium'>
          <Trans>Watched Addresses</Trans>
        </CardTitle>
        <CardDescription>
          <Trans>
            Coins and assets at these addresses show up in this wallet. Adding
            or removing an address doesn&apos;t require a resync.
          </Trans>
        </CardDescription>
      </CardHeader>
      <CardContent className='flex flex-col gap-4'>
        <div className='flex flex-col gap-3'>
          {addresses.map((address, index) => {
            const position = index + 1;

            return (
              <div key={address} className='flex items-end gap-2'>
                <AddressItem
                  address={address}
                  label={t`Address ${position}`}
                  className='flex-1 min-w-0'
                  truncateMiddle
                />
                <Button
                  variant='outline'
                  size='icon'
                  onClick={() => remove(address)}
                  disabled={pending || addresses.length <= 1}
                  aria-label={t`Stop watching ${address}`}
                  title={
                    addresses.length <= 1
                      ? t`A watch wallet needs at least one address`
                      : t`Stop watching this address`
                  }
                >
                  <TrashIcon className='h-4 w-4' aria-hidden='true' />
                </Button>
              </div>
            );
          })}
        </div>

        <div className='flex flex-col gap-2'>
          <Textarea
            className='resize-none h-20'
            placeholder={t`Addresses to watch, separated by spaces, commas or new lines`}
            aria-label={t`Addresses to watch`}
            value={input}
            onChange={(event) => setInput(event.target.value)}
          />
          <Button
            className='self-start'
            onClick={add}
            disabled={pending || toAdd.length === 0}
          >
            <PlusIcon className='h-4 w-4 mr-2' aria-hidden='true' />
            <Trans>Watch Addresses</Trans>
          </Button>
        </div>
      </CardContent>
    </Card>
  );
}

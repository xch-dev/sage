import Container from '@/components/Container';
import Header from '@/components/Header';
import { ReceiveAddress } from '@/components/ReceiveAddress';
import { Button } from '@/components/ui/button';
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card';
import { Switch } from '@/components/ui/switch';
import { useErrors } from '@/hooks/useErrors';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { useCallback, useEffect, useState } from 'react';
import { commands, events } from '../bindings';
import AddressList from '../components/AddressList';
import { useWalletState } from '../state';

export default function Addresses() {
  const { addError } = useErrors();
  const walletState = useWalletState();
  const ticker = walletState.sync.unit.ticker;

  const [hardened, setHardened] = useState(false);
  const [addresses, setAddresses] = useState<string[]>([]);

  const updateAddresses = useCallback(() => {
    commands
      .getDerivations({ offset: 0, limit: 1000000, hardened })
      .then((data) =>
        setAddresses(data.derivations.map((derivation) => derivation.address)),
      )
      .catch(addError);
  }, [addError, hardened]);

  useEffect(() => {
    updateAddresses();

    const unlisten = events.syncEvent.listen((event) => {
      if (event.payload.type === 'derivation') {
        updateAddresses();
      }
    });

    return () => {
      unlisten.then((u) => u());
    };
  }, [updateAddresses]);

  const derivationIndex = addresses.length;

  return (
    <>
      <Header title={t`Receive ${ticker}`} />

      <Container className='flex flex-col gap-4 max-w-screen-lg'>
        <Card>
          <CardHeader>
            <CardTitle className='text-lg font-medium'>
              <Trans>Fresh Address</Trans>
            </CardTitle>
            <CardDescription>
              <Trans>
                The wallet generates a new address after each transaction. Old
                ones stay valid.
              </Trans>
            </CardDescription>
          </CardHeader>
          <CardContent>
            <ReceiveAddress />
          </CardContent>
        </Card>
        <Card className='max-w-full'>
          <CardHeader>
            <CardTitle className='text-lg font-medium'>
              <Trans>All Addresses</Trans>
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className='flex items-center gap-2'>
              <label htmlFor='viewHidden'>
                <Trans>Hardened addresses</Trans>
              </label>
              <Switch
                id='hardened'
                checked={hardened}
                onCheckedChange={(value) => setHardened(value)}
              />
            </div>

            <div className='my-4 flex items-center gap-2'>
              <Trans>The current derivation index is {derivationIndex}</Trans>
              <Button variant='secondary' size='sm'>
                <Trans>Increase</Trans>
              </Button>
            </div>

            <AddressList addresses={addresses} />
          </CardContent>
        </Card>
      </Container>
    </>
  );
}

import Container from '@/components/Container';
import Header from '@/components/Header';
import { ReceiveAddress } from '@/components/ReceiveAddress';
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card';
import { useEffect, useState } from 'react';
import { commands, events } from '../bindings';
import AddressList from '../components/AddressList';
import { useWalletState } from '../state';

export default function Addresses() {
  const walletState = useWalletState();

  const [addresses, setAddresses] = useState<string[]>([]);

  console.log(addresses);

  const updateAddresses = () => {
    commands.getAddresses({}).then((res) => {
      if (res.status === 'error') return;
      setAddresses(res.data.addresses);
    });
  };

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
  }, []);

  return (
    <>
      <Header title={`Receive ${walletState.sync.unit.ticker}`} />

      <Container className='flex flex-col gap-4 max-w-screen-lg'>
        <Card>
          <CardHeader>
            <CardTitle className='text-lg font-medium'>Fresh Address</CardTitle>
            <CardDescription>
              The wallet generates a new address after each transaction. Old
              ones stay valid.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <ReceiveAddress />
          </CardContent>
        </Card>
        <Card className='max-w-full'>
          <CardHeader>
            <CardTitle className='text-lg font-medium'>All Addresses</CardTitle>
          </CardHeader>
          <CardContent>
            <AddressList addresses={addresses} />
          </CardContent>
        </Card>
      </Container>
    </>
  );
}

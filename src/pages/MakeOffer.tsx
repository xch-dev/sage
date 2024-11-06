import Container from '@/components/Container';
import Header from '@/components/Header';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { HandCoins, Handshake, PlusIcon } from 'lucide-react';

export function MakeOffer() {
  return (
    <>
      <Header title='Make Offer' />

      <Container>
        <div className='grid grid-cols-1 lg:grid-cols-2 gap-4'>
          <Card>
            <CardHeader className='flex flex-row items-center justify-between space-y-0 pb-2 pr-2 space-x-2'>
              <CardTitle className='text-md font-medium truncate flex items-center'>
                <HandCoins className='mr-2 h-4 w-4' />
                Offered Coins
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className='text-sm font-medium'>
                These are the coins that you are offering.
              </div>

              <Button variant='outline' size='icon' className='mt-4'>
                <PlusIcon className='h-4 w-4' />
              </Button>
            </CardContent>
          </Card>

          <Card>
            <CardHeader className='flex flex-row items-center justify-between space-y-0 pb-2 pr-2 space-x-2'>
              <CardTitle className='text-md font-medium truncate flex items-center'>
                <Handshake className='mr-2 h-4 w-4' />
                Requested Coins
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className='text-sm font-medium'>
                These coins will be received in exchange.
              </div>
            </CardContent>
          </Card>
        </div>

        <Button className='mt-4'>Make Offer</Button>
      </Container>
    </>
  );
}

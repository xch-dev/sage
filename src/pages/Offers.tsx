import Container from '@/components/Container';
import Header from '@/components/Header';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { HandCoins } from 'lucide-react';
import { Link, useNavigate } from 'react-router-dom';

export function Offers() {
  const navigate = useNavigate();

  return (
    <>
      <Header title='Offers' />

      <Container>
        <Card>
          <CardHeader className='flex flex-row items-center justify-between space-y-0 pb-2 pr-2 space-x-2'>
            <CardTitle className='text-md font-medium truncate flex items-center'>
              <HandCoins className='mr-2 h-4 w-4' />
              Offer Builder
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className='text-sm font-medium'>
              You can create an offer file that when taken will swap any number
              of tokens trustlessly without an intermediary. This file can be
              shared peer to peer or on decentralized exchanges like Dexie.
            </div>

            <Link to='/offers/make'>
              <Button className='mt-4'>Make Offer</Button>
            </Link>
          </CardContent>
        </Card>
      </Container>
    </>
  );
}

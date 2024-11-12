import Container from '@/components/Container';
import Header from '@/components/Header';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { useWalletState } from '@/state';
import { HandCoins } from 'lucide-react';
import { useEffect } from 'react';
import { Link, useNavigate } from 'react-router-dom';

export function Offers() {
  const navigate = useNavigate();
  const walletState = useWalletState();

  useEffect(() => {
    if (
      walletState.offerFee ||
      walletState.offerAssets.xch ||
      walletState.offerAssets.cats.length ||
      walletState.offerAssets.nfts.length ||
      walletState.requestedAssets.xch ||
      walletState.requestedAssets.cats.length ||
      walletState.requestedAssets.nfts.length
    ) {
      navigate('/offers/make', { replace: true });
    }
  }, [navigate, walletState]);

  return (
    <>
      <Header title='Offers' />

      <Container>
        <div className='grid grid-cols-1 gap-4 lg:grid-cols-2'>
          <Card>
            <CardHeader className='flex flex-row items-center justify-between space-y-0 pb-2 pr-2 space-x-2'>
              <CardTitle className='text-md font-medium truncate flex items-center'>
                <HandCoins className='mr-2 h-4 w-4' />
                Offer Builder
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className='text-sm font-medium'>
                You can create an offer file that when taken will swap any
                number of tokens trustlessly without an intermediary. This file
                can be shared peer to peer or on decentralized exchanges like
                Dexie.
              </div>

              <Link to='/offers/make'>
                <Button className='mt-4'>Make Offer</Button>
              </Link>
            </CardContent>
          </Card>

          <Card>
            <CardHeader className='flex flex-row items-center justify-between space-y-0 pb-2 pr-2 space-x-2'>
              <CardTitle className='text-md font-medium truncate flex items-center'>
                <HandCoins className='mr-2 h-4 w-4' />
                View Offer
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className='text-sm font-medium'>
                Enter a full offer file here to view information about the
                offer, including the offered and requested assets. You can
                decide whether to take the offer after reviewing its details.
              </div>

              <Input
                className='mt-4'
                placeholder='Paste offer file here'
                onChange={(e) => {
                  const offer = e.target.value.trim();
                  navigate(`/offers/view/${offer}`);
                }}
              />
            </CardContent>
          </Card>
        </div>
      </Container>
    </>
  );
}

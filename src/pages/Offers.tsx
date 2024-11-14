import Container from '@/components/Container';
import Header from '@/components/Header';
import { Button } from '@/components/ui/button';
import { useWalletState } from '@/state';
import { HandCoins } from 'lucide-react';
import { useCallback, useEffect } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog';
import { Textarea } from '@/components/ui/textarea';
import { useState } from 'react';

export function Offers() {
  const navigate = useNavigate();
  const walletState = useWalletState();
  const [offerString, setOfferString] = useState('');
  const [dialogOpen, setDialogOpen] = useState(false);

  const viewOffer = useCallback(
    (offer: string) => {
      if (offer.trim()) {
        navigate(`/offers/view/${encodeURIComponent(offer.trim())}`);
      }
    },
    [navigate],
  );

  useEffect(() => {
    const handlePaste = (e: ClipboardEvent) => {
      const text = e.clipboardData?.getData('text');
      if (text) {
        viewOffer(text);
      }
    };

    window.addEventListener('paste', handlePaste);
    return () => window.removeEventListener('paste', handlePaste);
  }, [viewOffer]);

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

  const handleViewOffer = (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    viewOffer(offerString);
  };

  return (
    <>
      <Header title='Offers'>
        <div className='flex items-center gap-2'>
          <Dialog open={dialogOpen} onOpenChange={setDialogOpen}>
            <DialogTrigger asChild>
              <Button variant='outline' className='flex items-center gap-1'>
                View offer
              </Button>
            </DialogTrigger>
            <DialogContent>
              <DialogHeader>
                <DialogTitle>Enter Offer String</DialogTitle>
              </DialogHeader>
              <form onSubmit={handleViewOffer} className='flex flex-col gap-4'>
                <Textarea
                  placeholder='Paste your offer string here...'
                  value={offerString}
                  onChange={(e) => setOfferString(e.target.value)}
                  className='min-h-[200px] font-mono text-xs'
                />
                <Button type='submit'>View Offer</Button>
              </form>
            </DialogContent>
          </Dialog>
          <Link to='/offers/make'>
            <Button>New offer</Button>
          </Link>
        </div>
      </Header>

      <Container>
        <div className='flex flex-col items-center justify-center py-12 text-center gap-4'>
          <HandCoins className='h-12 w-12 text-muted-foreground' />
          <div>
            <h2 className='text-lg font-semibold'>No Offers yet</h2>
            <p className='mt-2 text-sm text-muted-foreground'>
              Create a new offer to get started with peer-to-peer trading.
            </p>
            <p className='mt-1 text-sm text-muted-foreground'>
              You can also paste an offer using <kbd>Ctrl+V</kbd>.
            </p>
          </div>
          <Link to='/offers/make'>
            <Button>Create new offer</Button>
          </Link>
        </div>
      </Container>
    </>
  );
}

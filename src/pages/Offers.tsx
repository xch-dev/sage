import { commands, OfferRecord } from '@/bindings';
import Container from '@/components/Container';
import Header from '@/components/Header';
import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { Textarea } from '@/components/ui/textarea';
import { useWalletState } from '@/state';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import { CopyIcon, HandCoins, MoreVertical, TrashIcon } from 'lucide-react';
import { useCallback, useEffect, useState } from 'react';
import { Link, useNavigate } from 'react-router-dom';

export function Offers() {
  const navigate = useNavigate();
  const walletState = useWalletState();

  const [offerString, setOfferString] = useState('');
  const [dialogOpen, setDialogOpen] = useState(false);
  const [offers, setOffers] = useState<OfferRecord[]>([]);

  console.log(offers);

  const viewOffer = useCallback(
    (offer: string) => {
      if (offer.trim()) {
        navigate(`/offers/view/${encodeURIComponent(offer.trim())}`);
      }
    },
    [navigate],
  );

  useEffect(() => {
    commands.getOffers({}).then((result) => {
      if (result.status === 'error') {
        console.error(result.error);
        return;
      }

      setOffers(result.data.offers);
    });
  }, []);

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
      <Dialog open={dialogOpen} onOpenChange={setDialogOpen}>
        <Header title='Offers' />

        <Container>
          <div className='flex flex-col gap-10'>
            <div className='flex flex-col items-center justify-center pt-10 text-center gap-4'>
              <HandCoins className='h-12 w-12 text-muted-foreground' />
              <div>
                <h2 className='text-lg font-semibold'>
                  {offers.length > 0 ? 'Manage offers' : 'No offers yet'}
                </h2>
                <p className='mt-2 text-sm text-muted-foreground'>
                  Create a new offer to get started with peer-to-peer trading.
                </p>
                <p className='mt-1 text-sm text-muted-foreground'>
                  You can also paste an offer using <kbd>Ctrl+V</kbd>.
                </p>
              </div>
              <div className='flex gap-2'>
                <DialogTrigger asChild>
                  <Button variant='outline' className='flex items-center gap-1'>
                    View offer
                  </Button>
                </DialogTrigger>
                <Link to='/offers/make'>
                  <Button>Create offer</Button>
                </Link>
              </div>
            </div>

            <div className='flex flex-col gap-2'>
              {offers.map((record, i) => (
                <Offer
                  record={record}
                  refresh={() => {
                    commands.getOffers({}).then((result) => {
                      if (result.status === 'error') {
                        console.error(result.error);
                        return;
                      }
                      setOffers(result.data.offers);
                    });
                  }}
                  key={i}
                />
              ))}
            </div>
          </div>
        </Container>

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
    </>
  );
}

interface OfferProps {
  record: OfferRecord;
  refresh: () => void;
}

function Offer({ record, refresh }: OfferProps) {
  const [isDeleteOpen, setDeleteOpen] = useState(false);

  return (
    <>
      <Link
        to={`/offers/view/${encodeURIComponent(record.offer.trim())}`}
        className='block p-4 rounded-sm bg-neutral-100 dark:bg-neutral-900'
      >
        <div className='flex justify-between items-center -mt-1.5'>
          <span className='text-lg'>
            {record.status === 'active'
              ? 'Pending'
              : record.status === 'completed'
                ? 'Taken'
                : record.status === 'cancelled'
                  ? 'Cancelled'
                  : 'Expired'}
          </span>

          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button variant='ghost' size='icon' className='-mr-2'>
                <MoreVertical className='h-5 w-5' />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align='end'>
              <DropdownMenuGroup>
                <DropdownMenuItem
                  className='cursor-pointer'
                  onClick={(e) => {
                    e.stopPropagation();
                    writeText(record.offer);
                  }}
                >
                  <CopyIcon className='mr-2 h-4 w-4' />
                  <span>Copy</span>
                </DropdownMenuItem>

                <DropdownMenuItem
                  className='cursor-pointer'
                  onClick={(e) => {
                    e.stopPropagation();
                    setDeleteOpen(true);
                  }}
                >
                  <TrashIcon className='mr-2 h-4 w-4' />
                  <span>Delete</span>
                </DropdownMenuItem>
              </DropdownMenuGroup>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>

        <span className='text-muted-foreground text-sm'>
          {record.creation_date}
        </span>
      </Link>

      <Dialog open={isDeleteOpen} onOpenChange={setDeleteOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Delete offer record?</DialogTitle>
            <DialogDescription>
              This will delete the offer from the wallet, but if it's shared
              externally it can still be accepted. The only way to truly cancel
              a public offer is by spending one or more of its coins.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant='outline' onClick={() => setDeleteOpen(false)}>
              Cancel
            </Button>
            <Button
              onClick={() => {
                setDeleteOpen(false);
                commands
                  .deleteOffer({ offer_id: record.offer_id })
                  .then((result) => {
                    if (result.status === 'error') {
                      console.error(result.error);
                      return;
                    }
                    refresh();
                  });
              }}
            >
              Delete
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  );
}

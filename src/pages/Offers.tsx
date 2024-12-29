import {
  commands,
  events,
  OfferAssets,
  OfferRecord,
  TransactionResponse,
} from '@/bindings';
import ConfirmationDialog from '@/components/ConfirmationDialog';
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
import {
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from '@/components/ui/form';
import { TokenAmountInput } from '@/components/ui/masked-input';
import { Textarea } from '@/components/ui/textarea';
import { useErrors } from '@/hooks/useErrors';
import { amount } from '@/lib/formTypes';
import { nftUri } from '@/lib/nftUri';
import { toDecimal, toMojos } from '@/lib/utils';
import { isDefaultOffer, useOfferState, useWalletState } from '@/state';
import { zodResolver } from '@hookform/resolvers/zod';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import { platform } from '@tauri-apps/plugin-os';
import BigNumber from 'bignumber.js';
import {
  CircleOff,
  CopyIcon,
  HandCoins,
  MoreVertical,
  TrashIcon,
} from 'lucide-react';
import { useCallback, useEffect, useState } from 'react';
import { useForm } from 'react-hook-form';
import { Link, useNavigate } from 'react-router-dom';
import { z } from 'zod';

export function Offers() {
  const navigate = useNavigate();
  const offerState = useOfferState();

  const { addError } = useErrors();

  const [offerString, setOfferString] = useState('');
  const [dialogOpen, setDialogOpen] = useState(false);
  const [offers, setOffers] = useState<OfferRecord[]>([]);

  const viewOffer = useCallback(
    (offer: string) => {
      if (offer.trim()) {
        navigate(`/offers/view/${encodeURIComponent(offer.trim())}`);
      }
    },
    [navigate],
  );

  const updateOffers = useCallback(
    () =>
      commands
        .getOffers({})
        .then((data) => setOffers(data.offers))
        .catch(addError),
    [addError],
  );

  useEffect(() => {
    updateOffers();

    const unlisten = events.syncEvent.listen((data) => {
      if (data.payload.type === 'coin_state') {
        updateOffers();
      }
    });

    return () => {
      unlisten.then((u) => u());
    };
  }, [updateOffers]);

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
    if (!isDefaultOffer(offerState)) {
      navigate('/offers/make', { replace: true });
    }
  }, [navigate, offerState]);

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
                  You can also paste an offer using{' '}
                  <kbd>{platform() === 'macos' ? '⌘+V' : 'Ctrl+V'}</kbd>.
                </p>
              </div>
              <div className='flex gap-2'>
                <DialogTrigger asChild>
                  <Button variant='outline' className='flex items-center gap-1'>
                    View offer
                  </Button>
                </DialogTrigger>
                <Link to='/offers/make' replace={true}>
                  <Button>Create offer</Button>
                </Link>
              </div>
            </div>

            <div className='flex flex-col gap-2'>
              {offers.map((record, i) => (
                <Offer record={record} refresh={updateOffers} key={i} />
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
  const walletState = useWalletState();

  const { addError } = useErrors();

  const [isDeleteOpen, setDeleteOpen] = useState(false);
  const [isCancelOpen, setCancelOpen] = useState(false);
  const [fee, setFee] = useState<string>('');

  const cancelSchema = z.object({
    fee: amount(walletState.sync.unit.decimals).refine(
      (amount) => BigNumber(walletState.sync.balance).gte(amount || 0),
      'Not enough funds to cover the fee',
    ),
  });

  const cancelForm = useForm<z.infer<typeof cancelSchema>>({
    resolver: zodResolver(cancelSchema),
    defaultValues: {
      fee: '0',
    },
  });

  const cancelHandler = (values: z.infer<typeof cancelSchema>) => {
    commands
      .cancelOffer({
        offer_id: record.offer_id,
        fee: toMojos(values.fee, walletState.sync.unit.decimals),
      })
      .then((response) => setResponse(response))
      .catch(addError)
      .finally(() => setCancelOpen(false));
  };

  const [response, setResponse] = useState<TransactionResponse | null>(null);

  return (
    <>
      <Link
        to={`/offers/view_saved/${record.offer_id.trim()}`}
        className='block p-4 rounded-sm bg-neutral-100 dark:bg-neutral-900'
      >
        <div className='flex justify-between'>
          <div className='grid grid-cols-1 md:grid-cols-3 gap-4'>
            <div className='flex flex-col gap-1'>
              <div>
                {record.status === 'active'
                  ? 'Pending'
                  : record.status === 'completed'
                    ? 'Taken'
                    : record.status === 'cancelled'
                      ? 'Cancelled'
                      : 'Expired'}
              </div>
              <div className='text-muted-foreground text-sm'>
                {record.creation_date}
              </div>
            </div>

            <AssetPreview label='Offered' assets={record.summary.maker} />
            <AssetPreview label='Requested' assets={record.summary.taker} />
          </div>

          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button
                variant='ghost'
                size='icon'
                className='-mr-1.5 flex-shrink-0'
              >
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

                <DropdownMenuItem
                  className='cursor-pointer'
                  onClick={(e) => {
                    e.stopPropagation();
                    setCancelOpen(true);
                  }}
                  disabled={record.status !== 'active'}
                >
                  <CircleOff className='mr-2 h-4 w-4' />
                  <span>Cancel</span>
                </DropdownMenuItem>
              </DropdownMenuGroup>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
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
                commands
                  .deleteOffer({ offer_id: record.offer_id })
                  .then(() => refresh())
                  .catch(addError)
                  .finally(() => setDeleteOpen(false));
              }}
            >
              Delete
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog open={isCancelOpen} onOpenChange={setCancelOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Cancel offer?</DialogTitle>
            <DialogDescription>
              This will cancel the offer on-chain with a transaction, preventing
              it from being taken even if someone has the original offer file.
            </DialogDescription>
          </DialogHeader>
          <Form {...cancelForm}>
            <form
              onSubmit={cancelForm.handleSubmit(cancelHandler)}
              className='space-y-4'
            >
              <FormField
                control={cancelForm.control}
                name='fee'
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>Network Fee</FormLabel>
                    <FormControl>
                      <TokenAmountInput {...field} />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <DialogFooter className='gap-2'>
                <Button
                  type='button'
                  variant='outline'
                  onClick={() => setCancelOpen(false)}
                >
                  Cancel
                </Button>
                <Button type='submit'>Submit</Button>
              </DialogFooter>
            </form>
          </Form>
        </DialogContent>
      </Dialog>

      <ConfirmationDialog response={response} close={() => setResponse(null)} />
    </>
  );
}

interface AssetPreviewProps {
  label: string;
  assets: OfferAssets;
}

function AssetPreview({ label, assets }: AssetPreviewProps) {
  const walletState = useWalletState();

  return (
    <div className='flex flex-col gap-1 w-[125px] lg:w-[200px] xl:w-[300px]'>
      <div>{label}</div>
      {BigNumber(assets.xch.amount)
        .plus(assets.xch.royalty)
        .isGreaterThan(0) && (
        <div className='flex items-center gap-2'>
          <img src='https://icons.dexie.space/xch.webp' className='w-8 h-8' />

          <div className='text-sm text-muted-foreground truncate'>
            {toDecimal(
              BigNumber(assets.xch.amount).plus(assets.xch.royalty).toString(),
              walletState.sync.unit.decimals,
            )}{' '}
            {walletState.sync.unit.ticker}
          </div>
        </div>
      )}
      {Object.entries(assets.cats).map(([_assetId, cat]) => (
        <div className='flex items-center gap-2'>
          <img src={cat.icon_url!} className='w-8 h-8' />

          <div className='text-sm text-muted-foreground truncate'>
            {toDecimal(BigNumber(cat.amount).plus(cat.royalty).toString(), 3)}{' '}
            {cat.name ?? cat.ticker ?? 'Unknown'}
          </div>
        </div>
      ))}
      {Object.entries(assets.nfts).map(([_nftId, nft]) => (
        <div className='flex items-center gap-2'>
          <img
            src={nftUri(nft.image_mime_type, nft.image_data)}
            className='w-8 h-8'
          />

          <div className='text-sm text-muted-foreground truncate'>
            {nft.name ?? 'Unknown'}
          </div>
        </div>
      ))}
    </div>
  );
}

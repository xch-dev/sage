import { commands, events, OfferRecord, TransactionResponse } from '@/bindings';
import ConfirmationDialog from '@/components/ConfirmationDialog';
import Container from '@/components/Container';
import Header from '@/components/Header';
import { OfferSummaryCard } from '@/components/OfferSummaryCard';
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
import { toMojos } from '@/lib/utils';
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
  Tags,
  TrashIcon,
} from 'lucide-react';
import { useCallback, useEffect, useState } from 'react';
import { useForm } from 'react-hook-form';
import { Link, useNavigate } from 'react-router-dom';
import { Trans } from '@lingui/react/macro';
import { t } from '@lingui/core/macro';
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
        <Header title={<Trans>Offers</Trans>} />

        <Container>
          <div className='flex flex-col gap-10'>
            <div className='flex flex-col items-center justify-center pt-10 text-center gap-4'>
              <HandCoins className='h-12 w-12 text-muted-foreground' />
              <div>
                <h2 className='text-lg font-semibold'>
                  {offers.length > 0 ? (
                    <Trans>Manage offers</Trans>
                  ) : (
                    <Trans>No offers yet</Trans>
                  )}
                </h2>
                <p className='mt-2 text-sm text-muted-foreground'>
                  <Trans>
                    Create a new offer to get started with peer-to-peer trading.
                  </Trans>
                </p>
                <p className='mt-1 text-sm text-muted-foreground'>
                  <Trans>You can also paste an offer using</Trans>
                  <kbd>{platform() === 'macos' ? '⌘+V' : 'Ctrl+V'}</kbd>.
                </p>
              </div>
              <div className='flex gap-2'>
                <DialogTrigger asChild>
                  <Button variant='outline' className='flex items-center gap-1'>
                    <Trans>View Offer</Trans>
                  </Button>
                </DialogTrigger>
                <Link to='/offers/make' replace={true}>
                  <Button>
                    <Trans>Create Offer</Trans>
                  </Button>
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
            <DialogTitle>
              <Trans>Enter Offer String</Trans>
            </DialogTitle>
          </DialogHeader>
          <form onSubmit={handleViewOffer} className='flex flex-col gap-4'>
            <Textarea
              placeholder={t`Paste your offer string here...`}
              value={offerString}
              onChange={(e) => setOfferString(e.target.value)}
              className='min-h-[200px] font-mono text-xs'
            />
            <Button type='submit'>
              <Trans>View Offer</Trans>
            </Button>
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
      <Link to={`/offers/view_saved/${record.offer_id.trim()}`}>
        <OfferSummaryCard
          record={record}
          content={
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
                    <span>
                      <Trans>Copy</Trans>
                    </span>
                  </DropdownMenuItem>

                  <DropdownMenuItem
                    className='cursor-pointer'
                    onClick={(e) => {
                      e.stopPropagation();
                      setDeleteOpen(true);
                    }}
                  >
                    <TrashIcon className='mr-2 h-4 w-4' />
                    <span>
                      <Trans>Delete</Trans>
                    </span>
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
                    <Trans>Cancel</Trans>
                  </DropdownMenuItem>

                  <DropdownMenuItem
                    className='cursor-pointer'
                    onClick={(e) => {
                      e.stopPropagation();
                      writeText(record.offer_id);
                    }}
                  >
                    <Tags className='mr-2 h-4 w-4' />
                    <span>
                      <Trans>Copy ID</Trans>
                    </span>
                  </DropdownMenuItem>
                </DropdownMenuGroup>
              </DropdownMenuContent>
            </DropdownMenu>
          }
        />
      </Link>

      <Dialog open={isDeleteOpen} onOpenChange={setDeleteOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>
              <Trans>Delete offer record?</Trans>
            </DialogTitle>
            <DialogDescription>
              <Trans>
                This will delete the offer from the wallet, but if it's shared
                externally it can still be accepted. The only way to truly
                cancel a public offer is by spending one or more of its coins.
              </Trans>
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant='outline' onClick={() => setDeleteOpen(false)}>
              <Trans>Cancel</Trans>
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
              <Trans>Delete</Trans>
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

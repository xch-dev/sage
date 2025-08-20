import { commands, events, OfferRecord, TransactionResponse } from '@/bindings';
import ConfirmationDialog from '@/components/ConfirmationDialog';
import { CancelOfferConfirmation } from '@/components/confirmations/CancelOfferConfirmation';
import Container from '@/components/Container';
import { CancelOfferDialog } from '@/components/dialogs/CancelOfferDialog';
import { DeleteOfferDialog } from '@/components/dialogs/DeleteOfferDialog';
import { NfcScanDialog } from '@/components/dialogs/NfcScanDialog';
import { ViewOfferDialog } from '@/components/dialogs/ViewOfferDialog';
import Header from '@/components/Header';
import { OfferRowCard } from '@/components/OfferRowCard';
import { Button } from '@/components/ui/button';
import { Dialog, DialogTrigger } from '@/components/ui/dialog';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/components/ui/tooltip';
import { useErrors } from '@/hooks/useErrors';
import { useScannerOrClipboard } from '@/hooks/useScannerOrClipboard';
import { amount } from '@/lib/formTypes';
import { toMojos } from '@/lib/utils';
import { useOfferState, useWalletState } from '@/state';
import { zodResolver } from '@hookform/resolvers/zod';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { platform } from '@tauri-apps/plugin-os';
import BigNumber from 'bignumber.js';
import {
  CircleOff,
  FilterIcon,
  HandCoins,
  NfcIcon,
  ScanIcon,
  TrashIcon,
} from 'lucide-react';
import { useCallback, useEffect, useState } from 'react';
import { useForm } from 'react-hook-form';
import { Link, useNavigate } from 'react-router-dom';
import { getNdefPayloads, isNdefAvailable } from 'tauri-plugin-sage';
import { useLocalStorage } from 'usehooks-ts';
import { z } from 'zod';

const OFFER_FILTER_STORAGE_KEY = 'sage-offer-filter';

export function Offers() {
  const navigate = useNavigate();
  const offerState = useOfferState();
  const walletState = useWalletState();
  const { addError } = useErrors();
  const [offerString, setOfferString] = useState('');
  const [dialogOpen, setDialogOpen] = useState(false);
  const [offers, setOffers] = useState<OfferRecord[]>([]);
  const [isNfcAvailable, setIsNfcAvailable] = useState(false);
  const [showScanUi, setShowScanUi] = useState(false);
  const [statusFilter, setStatusFilter] = useLocalStorage(
    OFFER_FILTER_STORAGE_KEY,
    'all',
  );
  const [isCancelAllOpen, setIsCancelAllOpen] = useState(false);
  const [cancelAllResponse, setCancelAllResponse] =
    useState<TransactionResponse | null>(null);
  const [cancelAllFee, setCancelAllFee] = useState<string>('');
  const [isDeleteAllOpen, setIsDeleteAllOpen] = useState(false);
  const cancelAllSchema = z.object({
    fee: amount(walletState.sync.unit.precision).refine(
      (amount) => BigNumber(walletState.sync.balance).gte(amount || 0),
      t`Not enough funds to cover the fee`,
    ),
  });

  const cancelAllForm = useForm<z.infer<typeof cancelAllSchema>>({
    resolver: zodResolver(cancelAllSchema),
  });

  const viewOffer = useCallback(
    (offer: string) => {
      if (offer.trim()) {
        navigate(`/offers/view/${encodeURIComponent(offer.trim())}`);
      }
    },
    [navigate],
  );

  const { handleScanOrPaste } = useScannerOrClipboard((scanResValue) => {
    viewOffer(scanResValue);
  });

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
    if (offerState) {
      navigate('/offers/make', { replace: true });
    }
  }, [navigate, offerState]);

  const handleViewOffer = (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    viewOffer(offerString);
  };

  useEffect(() => {
    isNdefAvailable().then(setIsNfcAvailable);
  }, [addError]);

  const handleNfcScan = async () => {
    const isAndroid = platform() === 'android';

    if (isAndroid) setShowScanUi(true);

    const payloads = await getNdefPayloads()
      .catch((error) =>
        addError({ kind: 'internal', reason: `Failed to scan NFC: ${error}` }),
      )
      .finally(() => setShowScanUi(false));

    if (!payloads) return;

    const payload = payloads[0].slice(3);
    const array = new Uint8Array(payload);
    const text = new TextDecoder().decode(array);

    viewOffer(text);
  };

  const filteredOffers = offers.filter((offer) =>
    statusFilter === 'all' ? true : offer.status === statusFilter,
  );

  const handleDeleteAll = async () => {
    try {
      for (const offer of filteredOffers) {
        await commands.deleteOffer({ offer_id: offer.offer_id });
      }
      updateOffers();
      setIsDeleteAllOpen(false);
    } catch (error) {
      addError({
        kind: 'internal',
        reason: `Failed to delete offers: ${error}`,
      });
    }
  };

  const cancelAllHandler = (values: z.infer<typeof cancelAllSchema>) => {
    const fee = toMojos(values.fee, walletState.sync.unit.precision);
    const activeOffers = filteredOffers.filter(
      (offer) => offer.status === 'active',
    );

    // Store the fee for display in confirmation
    setCancelAllFee(`${values.fee} ${walletState.sync.unit.ticker}`);

    commands
      .cancelOffers({
        offer_ids: activeOffers.map((offer) => offer.offer_id),
        fee,
      })
      .then((response) => {
        setCancelAllResponse(response);
      })
      .catch(addError)
      .finally(() => {
        setIsCancelAllOpen(false);
      });
  };

  return (
    <>
      <Header
        title={<Trans>Offers</Trans>}
        mobileActionItems={
          <div className='flex items-center gap-2'>
            <Button size='icon' variant='ghost' onClick={handleScanOrPaste}>
              <ScanIcon className='h-5 w-5' aria-hidden='true' />
            </Button>
            <Button
              size='icon'
              variant='ghost'
              disabled={!isNfcAvailable}
              onClick={handleNfcScan}
            >
              <NfcIcon className='h-5 w-5 ' aria-hidden='true' />
            </Button>
          </div>
        }
      />
      <Container>
        <div className='flex flex-col gap-10'>
          <div className='flex flex-col items-center justify-center pt-10 text-center gap-4'>
            <HandCoins
              className='h-12 w-12 text-muted-foreground'
              aria-hidden='true'
            />
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
                <Trans>You can also paste an offer using</Trans>{' '}
                <kbd>{platform() === 'macos' ? '⌘+V' : 'Ctrl+V'}</kbd>.
              </p>
            </div>
            <div className='flex gap-2'>
              <Dialog open={dialogOpen} onOpenChange={setDialogOpen}>
                <DialogTrigger asChild>
                  <Button variant='outline' className='flex items-center gap-1'>
                    <Trans>View Offer</Trans>
                  </Button>
                </DialogTrigger>
                <ViewOfferDialog
                  open={dialogOpen}
                  onOpenChange={setDialogOpen}
                  offerString={offerString}
                  setOfferString={setOfferString}
                  onSubmit={handleViewOffer}
                />
              </Dialog>
              <Link to='/offers/make' replace={true}>
                <Button>
                  <Trans>Create Offer</Trans>
                </Button>
              </Link>
            </div>
          </div>

          {offers.length > 0 && (
            <div className='flex flex-col gap-4'>
              <div className='flex flex-row items-center justify-between gap-4'>
                <div className='flex items-center gap-2'>
                  <FilterIcon
                    className='h-4 w-4 text-muted-foreground'
                    aria-hidden='true'
                  />
                  <Select value={statusFilter} onValueChange={setStatusFilter}>
                    <SelectTrigger className='w-[180px]'>
                      <SelectValue placeholder='Filter by status' />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value='all'>All Offers</SelectItem>
                      <SelectItem value='active'>Active</SelectItem>
                      <SelectItem value='completed'>Completed</SelectItem>
                      <SelectItem value='cancelled'>Cancelled</SelectItem>
                      <SelectItem value='expired'>Expired</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
                <div className='flex items-center gap-2 min-w-fit'>
                  {filteredOffers.some(
                    (offer) => offer.status === 'active',
                  ) && (
                    <TooltipProvider>
                      <Tooltip>
                        <TooltipTrigger asChild>
                          <Button
                            variant='outline'
                            size='sm'
                            className='flex items-center gap-1'
                            onClick={() => setIsCancelAllOpen(true)}
                          >
                            <CircleOff className='h-4 w-4' aria-hidden='true' />
                            <span className='hidden sm:inline'>
                              <Trans>Cancel All Active</Trans>
                            </span>
                          </Button>
                        </TooltipTrigger>
                        <TooltipContent>
                          <Trans>Cancel All Active Offers</Trans>
                        </TooltipContent>
                      </Tooltip>
                    </TooltipProvider>
                  )}
                  {filteredOffers.length > 0 && (
                    <TooltipProvider>
                      <Tooltip>
                        <TooltipTrigger asChild>
                          <Button
                            variant='destructive'
                            size='sm'
                            className='flex items-center gap-1'
                            onClick={() => setIsDeleteAllOpen(true)}
                          >
                            <TrashIcon className='h-4 w-4' aria-hidden='true' />
                            <span className='hidden sm:inline'>
                              <Trans>Delete All</Trans>
                            </span>
                          </Button>
                        </TooltipTrigger>
                        <TooltipContent>
                          <Trans>Delete All Filtered Offers</Trans>
                        </TooltipContent>
                      </Tooltip>
                    </TooltipProvider>
                  )}
                </div>
              </div>

              <div className='flex flex-col gap-2'>
                {filteredOffers.map((record) => (
                  <OfferRowCard
                    record={record}
                    key={record.offer_id}
                    refresh={updateOffers}
                  />
                ))}
              </div>
            </div>
          )}
        </div>
      </Container>

      <NfcScanDialog open={showScanUi} onOpenChange={setShowScanUi} />

      <DeleteOfferDialog
        open={isDeleteAllOpen}
        onOpenChange={setIsDeleteAllOpen}
        onDelete={handleDeleteAll}
        offerCount={filteredOffers.length}
      />

      <CancelOfferDialog
        open={isCancelAllOpen}
        onOpenChange={setIsCancelAllOpen}
        form={cancelAllForm}
        onSubmit={cancelAllHandler}
        title={<Trans>Cancel all active offers?</Trans>}
        description={
          <Trans>
            This will cancel all{' '}
            {filteredOffers.filter((offer) => offer.status === 'active').length}{' '}
            active offers on-chain with transactions, preventing them from being
            taken even if someone has the original offer files.
          </Trans>
        }
        feeLabel={<Trans>Network Fee (per offer)</Trans>}
      />

      <ConfirmationDialog
        response={cancelAllResponse}
        showRecipientDetails={false}
        close={() => {
          setCancelAllResponse(null);
          setCancelAllFee('');
        }}
        onConfirm={updateOffers}
        additionalData={{
          title: t`Cancel All Active Offers`,
          content: cancelAllResponse && (
            <CancelOfferConfirmation
              offers={filteredOffers.filter(
                (offer) => offer.status === 'active',
              )}
              fee={cancelAllFee}
            />
          ),
        }}
      />
    </>
  );
}

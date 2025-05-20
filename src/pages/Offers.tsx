import { commands, events, OfferRecord } from '@/bindings';
import Container from '@/components/Container';
import { NfcScanDialog } from '@/components/dialogs/NfcScanDialog';
import { ViewOfferDialog } from '@/components/dialogs/ViewOfferDialog';
import Header from '@/components/Header';
import { Button } from '@/components/ui/button';
import { Dialog, DialogTrigger } from '@/components/ui/dialog';
import { useErrors } from '@/hooks/useErrors';
import { useScannerOrClipboard } from '@/hooks/useScannerOrClipboard';
import { useOfferState } from '@/state';
import { Trans } from '@lingui/react/macro';
import { isAvailable, scan } from '@tauri-apps/plugin-nfc';
import { platform } from '@tauri-apps/plugin-os';
import {
  HandCoins,
  NfcIcon,
  ScanIcon,
  FilterIcon,
  TrashIcon,
  CircleOff,
} from 'lucide-react';
import { useCallback, useEffect, useState } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import { OfferRowCard } from '@/components/OfferRowCard';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from '@/components/ui/alert-dialog';
import { useLocalStorage } from 'usehooks-ts';

const OFFER_FILTER_STORAGE_KEY = 'sage-offer-filter';

export function Offers() {
  const navigate = useNavigate();
  const offerState = useOfferState();
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
    isAvailable()
      .then((available) => setIsNfcAvailable(available))
      .catch((error) => {
        console.warn('NFC not available:', error);
        setIsNfcAvailable(false);
      });
  }, []);

  const handleNfcScan = async () => {
    const isAndroid = platform() === 'android';

    if (isAndroid) setShowScanUi(true);

    const tag = await scan(
      { type: 'ndef' },
      {
        keepSessionAlive: false,
        message: 'Scan an NFC tag',
        successMessage: 'NFC tag successfully scanned',
      },
    )
      .catch((error) =>
        addError({ kind: 'internal', reason: `Failed to scan NFC: ${error}` }),
      )
      .finally(() => setShowScanUi(false));

    if (!tag) return;

    const payload = tag.records[0].payload.slice(3);
    const array = new Uint8Array(payload);
    const text = new TextDecoder().decode(array);

    viewOffer(text);
  };

  const filteredOffers = offers.filter((offer) =>
    statusFilter === 'all' ? true : offer.status === statusFilter,
  );

  const handleDeleteAll = async () => {
    try {
      await Promise.all(
        filteredOffers.map((offer) =>
          commands.deleteOffer({ offer_id: offer.offer_id }),
        ),
      );
      updateOffers();
    } catch (error) {
      addError({
        kind: 'internal',
        reason: `Failed to delete offers: ${error}`,
      });
    }
  };

  const handleCancelAll = async () => {
    try {
      await Promise.all(
        filteredOffers
          .filter((offer) => offer.status === 'active')
          .map((offer) =>
            commands.cancelOffer({
              offer_id: offer.offer_id,
              fee: '0',
            }),
          ),
      );
      updateOffers();
    } catch (error) {
      addError({
        kind: 'internal',
        reason: `Failed to cancel offers: ${error}`,
      });
    }
  };

  return (
    <>
      <Header
        title={<Trans>Offers</Trans>}
        mobileActionItems={
          <div className='flex items-center gap-2'>
            <Button size='icon' variant='ghost' onClick={handleScanOrPaste}>
              <ScanIcon className='h-5 w-5' />
            </Button>
            <Button
              size='icon'
              variant='ghost'
              disabled={!isNfcAvailable}
              onClick={handleNfcScan}
            >
              <NfcIcon className='h-5 w-5 ' />
            </Button>
          </div>
        }
      />
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
              <div className='flex items-center justify-between'>
                <div className='flex items-center gap-2'>
                  <FilterIcon className='h-4 w-4 text-muted-foreground' />
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
                <div className='flex gap-2'>
                  {filteredOffers.some(
                    (offer) => offer.status === 'active',
                  ) && (
                    <AlertDialog>
                      <AlertDialogTrigger asChild>
                        <Button
                          variant='outline'
                          size='sm'
                          className='flex items-center gap-1'
                        >
                          <CircleOff className='h-4 w-4' />
                          <Trans>Cancel All Active</Trans>
                        </Button>
                      </AlertDialogTrigger>
                      <AlertDialogContent>
                        <AlertDialogHeader>
                          <AlertDialogTitle>
                            Cancel All Active Offers
                          </AlertDialogTitle>
                          <AlertDialogDescription>
                            Are you sure you want to cancel all active offers?
                            This action cannot be undone.
                          </AlertDialogDescription>
                        </AlertDialogHeader>
                        <AlertDialogFooter>
                          <AlertDialogCancel>Cancel</AlertDialogCancel>
                          <AlertDialogAction onClick={handleCancelAll}>
                            Continue
                          </AlertDialogAction>
                        </AlertDialogFooter>
                      </AlertDialogContent>
                    </AlertDialog>
                  )}
                  {filteredOffers.length > 0 && (
                    <AlertDialog>
                      <AlertDialogTrigger asChild>
                        <Button
                          variant='destructive'
                          size='sm'
                          className='flex items-center gap-1'
                        >
                          <TrashIcon className='h-4 w-4' />
                          <Trans>Delete All</Trans>
                        </Button>
                      </AlertDialogTrigger>
                      <AlertDialogContent>
                        <AlertDialogHeader>
                          <AlertDialogTitle>
                            Delete All Filtered Offers
                          </AlertDialogTitle>
                          <AlertDialogDescription>
                            Are you sure you want to delete all{' '}
                            {filteredOffers.length} filtered offers? This action
                            cannot be undone.
                          </AlertDialogDescription>
                        </AlertDialogHeader>
                        <AlertDialogFooter>
                          <AlertDialogCancel>Cancel</AlertDialogCancel>
                          <AlertDialogAction onClick={handleDeleteAll}>
                            Continue
                          </AlertDialogAction>
                        </AlertDialogFooter>
                      </AlertDialogContent>
                    </AlertDialog>
                  )}
                </div>
              </div>

              <div className='flex flex-col gap-2'>
                {filteredOffers.map((record, i) => (
                  <OfferRowCard
                    record={record}
                    refresh={updateOffers}
                    key={i}
                  />
                ))}
              </div>
            </div>
          )}
        </div>
      </Container>

      <NfcScanDialog open={showScanUi} onOpenChange={setShowScanUi} />
    </>
  );
}

import { commands, events, OfferRecord } from '@/bindings';
import Container from '@/components/Container';
import { DeleteOfferDialog } from '@/components/dialogs/DeleteOfferDialog';
import { NfcScanDialog } from '@/components/dialogs/NfcScanDialog';
import { ViewOfferDialog } from '@/components/dialogs/ViewOfferDialog';
import Header from '@/components/Header';
import { OfferRowCard } from '@/components/OfferRowCard';
import { OffersMultiSelectActions } from '@/components/OffersMultiSelectActions';
import { ReadOnlyButton } from '@/components/ReadOnlyButton';
import { Button } from '@/components/ui/button';
import { Card } from '@/components/ui/card';
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
import { deleteOffers } from '@/lib/offers';
import { cn } from '@/lib/utils';
import { useOfferState } from '@/state';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { platform } from '@tauri-apps/plugin-os';
import {
  CopyPlus,
  FilterIcon,
  HandCoins,
  ImageIcon,
  NfcIcon,
  ScanIcon,
  TrashIcon,
} from 'lucide-react';
import { useCallback, useEffect, useRef, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { getNdefPayloads, isNdefAvailable } from 'tauri-plugin-sage';
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
  const [multiSelect, setMultiSelect] = useState(false);
  const [selected, setSelected] = useState<string[]>([]);
  const [isDeleteAllOpen, setIsDeleteAllOpen] = useState(false);

  const toggleSelected = useCallback((offerId: string, value: boolean) => {
    setSelected((prev) =>
      value ? [...prev, offerId] : prev.filter((id) => id !== offerId),
    );
  }, []);

  const viewOffer = useCallback(
    (offer: string) => {
      if (offer.trim()) {
        navigate(`/offers/view/${encodeURIComponent(offer.trim())}`);
      }
    },
    [navigate],
  );

  const { handleScanOrPaste, handleScanImage } = useScannerOrClipboard(
    (scanResValue) => {
      viewOffer(scanResValue);
    },
  );

  const isMobile = platform() === 'ios' || platform() === 'android';

  const offerImageInputRef = useRef<HTMLInputElement>(null);

  const handleOfferImageChange = (
    event: React.ChangeEvent<HTMLInputElement>,
  ) => {
    const file = event.target.files?.[0];
    event.target.value = '';
    if (file) {
      handleScanImage(file);
    }
  };

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
    if (platform() !== 'ios' && platform() !== 'android') return;
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

  useEffect(() => {
    setMultiSelect(false);
    setSelected([]);
  }, [statusFilter]);

  const scanImageButton = (
    <Button
      size='icon'
      variant='ghost'
      aria-label={t`Scan an offer QR code from an image`}
      onClick={() => offerImageInputRef.current?.click()}
    >
      <ImageIcon className='h-5 w-5' aria-hidden='true' />
    </Button>
  );

  return (
    <>
      <input
        ref={offerImageInputRef}
        type='file'
        accept='image/*'
        className='hidden'
        onChange={handleOfferImageChange}
      />
      <Header
        title={<Trans>Offers</Trans>}
        alwaysShowChildren
        mobileActionItems={
          <div className='flex items-center gap-2'>
            <Button
              size='icon'
              variant='ghost'
              aria-label={t`Scan an offer QR code with the camera`}
              onClick={handleScanOrPaste}
            >
              <ScanIcon className='h-5 w-5' aria-hidden='true' />
            </Button>
            {scanImageButton}
            <Button
              size='icon'
              variant='ghost'
              aria-label={t`Scan an offer from an NFC tag`}
              disabled={!isNfcAvailable}
              onClick={handleNfcScan}
            >
              <NfcIcon className='h-5 w-5 ' aria-hidden='true' />
            </Button>
          </div>
        }
      >
        {!isMobile && scanImageButton}
      </Header>
      <Container>
        <Card className='p-6'>
          <div className='flex flex-col gap-10'>
            <div className='flex flex-col items-center justify-center pt-4 text-center gap-4'>
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
                    <Button
                      variant='outline'
                      className='flex items-center gap-1'
                    >
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
                <ReadOnlyButton
                  requiresSigning
                  onClick={() => navigate('/offers/make', { replace: true })}
                >
                  <Trans>Create Offer</Trans>
                </ReadOnlyButton>
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
                    <Select
                      value={statusFilter}
                      onValueChange={setStatusFilter}
                    >
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
                    {filteredOffers.length > 0 && (
                      <TooltipProvider>
                        <Tooltip>
                          <TooltipTrigger asChild>
                            <Button
                              variant='outline'
                              size='icon'
                              onClick={() => {
                                setMultiSelect(!multiSelect);
                                setSelected([]);
                              }}
                              aria-label={t`Toggle multi-select`}
                            >
                              <CopyPlus
                                className={cn(
                                  'h-4 w-4',
                                  multiSelect &&
                                    'text-green-600 dark:text-green-400',
                                )}
                                aria-hidden='true'
                              />
                            </Button>
                          </TooltipTrigger>
                          <TooltipContent>
                            <Trans>Toggle multi-select</Trans>
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
                              <TrashIcon
                                className='h-4 w-4'
                                aria-hidden='true'
                              />
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
                      selectionState={
                        multiSelect
                          ? [
                              selected.includes(record.offer_id),
                              (value) => toggleSelected(record.offer_id, value),
                            ]
                          : null
                      }
                    />
                  ))}
                </div>
              </div>
            )}
          </div>
        </Card>
      </Container>

      <NfcScanDialog open={showScanUi} onOpenChange={setShowScanUi} />

      <DeleteOfferDialog
        open={isDeleteAllOpen}
        onOpenChange={setIsDeleteAllOpen}
        offerCount={filteredOffers.length}
        onDelete={() => {
          deleteOffers(filteredOffers.map((offer) => offer.offer_id))
            .then(updateOffers)
            .catch(addError)
            .finally(() => setIsDeleteAllOpen(false));
        }}
      />

      {selected.length > 0 && (
        <OffersMultiSelectActions
          selected={selected}
          offers={filteredOffers}
          onConfirm={() => {
            updateOffers();
            setSelected([]);
            setMultiSelect(false);
          }}
          onSelectAll={() =>
            setSelected(filteredOffers.map((offer) => offer.offer_id))
          }
          onClearSelection={() => setSelected([])}
        />
      )}
    </>
  );
}

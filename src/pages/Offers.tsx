import Container from '@/components/Container';
import { NfcScanDialog } from '@/components/dialogs/NfcScanDialog';
import { ViewOfferDialog } from '@/components/dialogs/ViewOfferDialog';
import Header from '@/components/Header';
import { OfferOptions } from '@/components/OfferOptions';
import { OfferRowCard } from '@/components/OfferRowCard';
import { OffersMultiSelectActions } from '@/components/OffersMultiSelectActions';
import { Pagination } from '@/components/Pagination';
import { ReadOnlyButton } from '@/components/ReadOnlyButton';
import { Button } from '@/components/ui/button';
import { Card } from '@/components/ui/card';
import { Dialog, DialogTrigger } from '@/components/ui/dialog';
import { useErrors } from '@/hooks/useErrors';
import { useIntersectionObserver } from '@/hooks/useIntersectionObserver';
import { useMultiSelect } from '@/hooks/useMultiSelect';
import { useOfferData } from '@/hooks/useOfferData';
import { useOfferParams } from '@/hooks/useOfferParams';
import { useScannerOrClipboard } from '@/hooks/useScannerOrClipboard';
import { cn } from '@/lib/utils';
import { useOfferState } from '@/state';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { platform } from '@tauri-apps/plugin-os';
import {
  HandCoins,
  ImageIcon,
  NfcIcon,
  PlusIcon,
  ScanIcon,
} from 'lucide-react';
import { useCallback, useEffect, useRef, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { getNdefPayloads, isNdefAvailable } from 'tauri-plugin-sage';

export function Offers() {
  const navigate = useNavigate();
  const offerState = useOfferState();
  const { addError } = useErrors();
  const [offerString, setOfferString] = useState('');
  const [dialogOpen, setDialogOpen] = useState(false);
  const [isNfcAvailable, setIsNfcAvailable] = useState(false);
  const [showScanUi, setShowScanUi] = useState(false);

  const [params, setParams] = useOfferParams();
  const { offers, total, isLoading, loaded, isPastEnd, isCurrent, refresh } =
    useOfferData(params);
  const {
    multiSelect,
    setMultiSelect,
    selected,
    selectionStateFor,
    selectAll,
    clear,
  } = useMultiSelect();

  const optionsRef = useRef<HTMLDivElement>(null);
  const [isOptionsVisible, setIsOptionsVisible] = useState(true);

  useIntersectionObserver(optionsRef, ([entry]) => {
    setIsOptionsVisible(entry.isIntersecting);
  });

  // Deleting or cancelling the last offers on the final page leaves it empty.
  // A stale `?page=N` URL (e.g. from history/back-forward) can also land far
  // past the actual last page; jump straight to page 1 rather than walking
  // back one page (and one request) at a time.
  useEffect(() => {
    if (isPastEnd) {
      setParams({ page: 1 });
    }
  }, [isPastEnd, setParams]);

  const hasFilters = params.query !== null || params.status !== 'all';
  // Gated on `isCurrent` so these don't flash stale states: e.g. clicking
  // "Clear filters" on a no-match result shouldn't show "No offers yet"
  // using the previous (filtered, empty) response while the unfiltered one
  // is still loading.
  const showIntro =
    loaded && isCurrent && total === 0 && params.page === 1 && !hasFilters;
  const showNoMatches =
    loaded &&
    isCurrent &&
    offers.length === 0 &&
    params.page === 1 &&
    hasFilters;

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

  const renderPagination = useCallback(
    (compact = false) => (
      <Pagination
        page={params.page}
        total={total}
        pageSize={params.pageSize}
        onPageChange={(page) => setParams({ page })}
        onPageSizeChange={(pageSize) => setParams({ pageSize, page: 1 })}
        pageSizeOptions={[24, 48, 72, 96]}
        compact={compact}
        canLoadMore={offers.length === params.pageSize}
        isLoading={isLoading}
      />
    ),
    [params.page, params.pageSize, total, setParams, offers.length, isLoading],
  );

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

  const actionButtons = (
    <div className='flex gap-2'>
      <ReadOnlyButton
        requiresSigning
        onClick={() => navigate('/offers/make', { replace: true })}
      >
        <PlusIcon className='h-4 w-4 mr-2' aria-hidden='true' />
        <Trans>Create Offer</Trans>
      </ReadOnlyButton>
      <Dialog open={dialogOpen} onOpenChange={setDialogOpen}>
        <DialogTrigger asChild>
          <Button variant='outline'>
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
    </div>
  );

  const pasteShortcut = platform() === 'macos' ? '⌘+V' : 'Ctrl+V';

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
        paginationControls={
          !showIntro && !isOptionsVisible ? renderPagination(true) : undefined
        }
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
        {showIntro ? (
          <Card className='p-6'>
            <div className='flex flex-col items-center justify-center pt-4 text-center gap-4'>
              <HandCoins
                className='h-12 w-12 text-muted-foreground'
                aria-hidden='true'
              />
              <div>
                <h2 className='text-lg font-semibold'>
                  <Trans>No offers yet</Trans>
                </h2>
                <p className='mt-2 text-sm text-muted-foreground'>
                  <Trans>
                    Create a new offer to get started with peer-to-peer trading.
                  </Trans>
                </p>
                <p className='mt-1 text-sm text-muted-foreground'>
                  <Trans>You can also paste an offer using</Trans>{' '}
                  <kbd>{pasteShortcut}</kbd>.
                </p>
              </div>
              {actionButtons}
            </div>
          </Card>
        ) : (
          <div className='flex flex-wrap items-center gap-x-4 gap-y-2'>
            {actionButtons}
            <span className='text-sm text-muted-foreground'>
              <Trans>or paste an offer with</Trans> <kbd>{pasteShortcut}</kbd>
            </span>
          </div>
        )}

        {/* Always mounted: useIntersectionObserver binds to this element once. */}
        <div ref={optionsRef} className={cn(showIntro && 'hidden')}>
          <OfferOptions
            params={params}
            setParams={setParams}
            multiSelect={multiSelect}
            setMultiSelect={setMultiSelect}
            renderPagination={() => renderPagination(false)}
            className='mt-4'
          />
        </div>

        {!showIntro && (
          <main aria-label={t`Offers`} className='mt-4 flex flex-col gap-2'>
            {showNoMatches ? (
              <div className='flex flex-col items-center gap-3 py-10 text-center text-sm text-muted-foreground'>
                <p>
                  <Trans>No offers match your filters.</Trans>
                </p>
                <Button
                  variant='outline'
                  size='sm'
                  onClick={() =>
                    setParams({
                      query: null,
                      status: 'all',
                      findSide: 'any',
                      page: 1,
                    })
                  }
                >
                  <Trans>Clear filters</Trans>
                </Button>
              </div>
            ) : (
              offers.map((record) => (
                <OfferRowCard
                  record={record}
                  key={record.offer_id}
                  refresh={refresh}
                  selectionState={selectionStateFor(record.offer_id)}
                />
              ))
            )}
          </main>
        )}
      </Container>

      <NfcScanDialog open={showScanUi} onOpenChange={setShowScanUi} />

      {selected.length > 0 && (
        <OffersMultiSelectActions
          selected={selected}
          onConfirm={() => {
            refresh();
            setMultiSelect(false);
          }}
          onSelectAll={() => selectAll(offers.map((offer) => offer.offer_id))}
          onClearSelection={clear}
        />
      )}
    </>
  );
}

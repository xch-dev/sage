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
import { HandCoins, NfcIcon, ScanIcon } from 'lucide-react';
import { useCallback, useEffect, useState } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import { OfferRowCard } from '@/components/OfferRowCard';

export function Offers() {
  const navigate = useNavigate();
  const offerState = useOfferState();
  const { addError } = useErrors();
  const [offerString, setOfferString] = useState('');
  const [dialogOpen, setDialogOpen] = useState(false);
  const [offers, setOffers] = useState<OfferRecord[]>([]);
  const [isNfcAvailable, setIsNfcAvailable] = useState(false);
  const [showScanUi, setShowScanUi] = useState(false);

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
    let text = new TextDecoder().decode(array);

    if (!text.startsWith('DT001')) {
      addError({
        kind: 'nfc',
        reason: 'Invalid NFC payload (not following CHIP-0047)',
      });
      return;
    }

    text = text.slice(5);

    const nftId = text.slice(0, 62);

    if (nftId.length !== 62 || !nftId.startsWith('nft1')) {
      addError({
        kind: 'nfc',
        reason:
          'NFC payload starts with CHIP-0047 prefix but does not have a valid NFT ID',
      });
      return;
    }

    text = text.slice(62);

    const offer = await commands.downloadCniOffercode(text).catch(addError);

    if (!offer) return;

    viewOffer(offer);
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

          <div className='flex flex-col gap-2'>
            {offers.map((record, i) => (
              <OfferRowCard record={record} refresh={updateOffers} key={i} />
            ))}
          </div>
        </div>
      </Container>

      <NfcScanDialog open={showScanUi} onOpenChange={setShowScanUi} />
    </>
  );
}

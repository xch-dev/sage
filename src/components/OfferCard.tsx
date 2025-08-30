import {
  commands,
  NetworkKind,
  OfferRecordStatus,
  OfferSummary,
} from '@/bindings';
import { Assets } from '@/components/Assets';
import { MarketplaceCard } from '@/components/MarketplaceCard';
import { NumberFormat } from '@/components/NumberFormat';
import { marketplaces } from '@/lib/marketplaces';
import { formatTimestamp, fromMojos } from '@/lib/utils';
import { useWalletState } from '@/state';
import { shareText } from '@buildyourwebapp/tauri-plugin-sharesheet';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { platform } from '@tauri-apps/plugin-os';
import {
  Calendar,
  Clock,
  Copy,
  HandCoinsIcon,
  InfoIcon,
  Share,
  ShoppingBasketIcon,
  Tags,
} from 'lucide-react';
import { useEffect, useState } from 'react';
import { toast } from 'react-toastify';
import { LabeledItem } from './LabeledItem';
import { Card, CardContent, CardHeader, CardTitle } from './ui/card';

// Interface to track CAT presence in wallet
type CatPresence = Record<string, boolean>;

interface OfferCardProps {
  offerId?: string;
  offer?: string;
  status?: OfferRecordStatus;
  creationTimestamp?: number;
  summary?: OfferSummary;
  content?: React.ReactNode;
}

export function OfferCard({
  offerId,
  offer,
  status,
  creationTimestamp,
  summary: offerSummary,
  content,
}: OfferCardProps) {
  const walletState = useWalletState();
  // State to track which CATs are present in the wallet
  const [catPresence, setCatPresence] = useState<CatPresence>({});
  const [network, setNetwork] = useState<NetworkKind | null>(null);
  const isMobile = platform() === 'ios' || platform() === 'android';

  const handleShare = async () => {
    if (!offer) return;

    try {
      await shareText(offer, {
        title: t`Offer`,
        mimeType: 'text/plain',
      });
    } catch (error: unknown) {
      toast.error(`${error instanceof Error ? error.message : String(error)}`);
    }
  };

  const handleCopy = async () => {
    if (!offer) return;

    try {
      await navigator.clipboard.writeText(offer);
      toast.success(t`Offer copied to clipboard`);
    } catch (error: unknown) {
      toast.error(`${error instanceof Error ? error.message : String(error)}`);
    }
  };

  useEffect(() => {
    commands.getCats({}).then((data) => {
      const presence: CatPresence = {};
      data.cats.forEach((cat) => {
        presence[cat.asset_id ?? ''] = true;
      });
      setCatPresence(presence);
    });
  }, []);

  useEffect(() => {
    commands.getNetwork({}).then((data) => setNetwork(data.kind));
  }, []);

  if (!offerSummary) return null;

  return (
    <div className='flex flex-col gap-4 max-w-screen-lg pr-1'>
      <Card>
        <CardHeader className='pb-2'>
          <CardTitle className='flex items-center justify-between'>
            <div className='flex items-center gap-2'>
              <InfoIcon className='h-5 w-5' />
              <Trans>Offer Details</Trans>
            </div>
            {offer && (
              <div className='flex items-center gap-2'>
                {!isMobile && (
                  <button
                    type='button'
                    onClick={handleCopy}
                    className='flex items-center gap-2 px-3 py-1.5 rounded-md border hover:bg-accent w-fit'
                    title={t`Copy offer`}
                  >
                    <Copy className='h-4 w-4' aria-hidden='true' />
                  </button>
                )}
                {isMobile && (
                  <button
                    type='button'
                    onClick={handleShare}
                    className='flex items-center gap-2 px-3 py-1.5 rounded-md border hover:bg-accent w-fit'
                    title={t`Share offer`}
                  >
                    <Share className='h-4 w-4' aria-hidden='true' />
                  </button>
                )}
              </div>
            )}
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className='flex items-center gap-4 mb-4'>
            {status && (
              <div
                className={`px-3 py-1 rounded-full text-sm font-medium ${
                  status === 'active' || status === 'pending'
                    ? 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-300'
                    : status === 'completed'
                      ? 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-300'
                      : status === 'cancelled'
                        ? 'bg-amber-100 text-amber-800 dark:bg-amber-900 dark:text-amber-300'
                        : 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-300'
                }`}
              >
                {status === 'active' || status === 'pending'
                  ? t`Pending`
                  : status === 'completed'
                    ? t`Taken`
                    : status === 'cancelled'
                      ? t`Cancelled`
                      : t`Expired`}
              </div>
            )}
          </div>

          <div className='grid grid-cols-1 md:grid-cols-3 gap-4'>
            {creationTimestamp && (
              <div className='flex items-center gap-2'>
                <Calendar className='h-4 w-4 text-muted-foreground' />
                <LabeledItem
                  label={t`Created`}
                  content={formatTimestamp(creationTimestamp, 'short', 'short')}
                />
              </div>
            )}

            {(offerSummary.expiration_timestamp ||
              offerSummary.expiration_height) && (
              <div className='flex items-center gap-2'>
                <Clock className='h-4 w-4 text-muted-foreground' />
                <LabeledItem
                  label={t`Expires`}
                  content={
                    offerSummary.expiration_timestamp
                      ? formatTimestamp(
                          offerSummary.expiration_timestamp,
                          'short',
                          'short',
                        )
                      : offerSummary.expiration_height
                        ? offerSummary.expiration_height?.toString()
                        : null
                  }
                />
              </div>
            )}

            <LabeledItem label={t`Maker Fee`} content={null}>
              <div className='text-sm'>
                <NumberFormat
                  value={fromMojos(
                    offerSummary.fee,
                    walletState.sync.unit.precision,
                  )}
                  minimumFractionDigits={0}
                  maximumFractionDigits={walletState.sync.unit.precision}
                />{' '}
                {walletState.sync.unit.ticker}
              </div>
            </LabeledItem>
          </div>
        </CardContent>
      </Card>

      <div className='grid grid-cols-1 lg:grid-cols-2 gap-4'>
        <Card>
          <CardHeader className='pb-2'>
            <CardTitle className='text-lg font-medium flex items-center'>
              <ShoppingBasketIcon className='mr-2 h-5 w-5' aria-hidden='true' />
              <Trans>Requested</Trans>
            </CardTitle>
            <p className='text-sm text-muted-foreground'>
              <Trans>
                The assets the taker will have to pay to fulfill the offer.
              </Trans>
            </p>
          </CardHeader>
          <CardContent className='flex flex-col gap-3'>
            <Assets assets={offerSummary.taker} />
            {content}
          </CardContent>
        </Card>

        <Card>
          <CardHeader className='pb-2'>
            <CardTitle className='text-lg font-medium flex items-center'>
              <HandCoinsIcon className='mr-2 h-5 w-5' />
              <Trans>Offered</Trans>
            </CardTitle>
            <p className='text-sm text-muted-foreground'>
              <Trans>The assets being given to the taker in the offer.</Trans>
            </p>
          </CardHeader>
          <CardContent className='flex flex-col gap-3'>
            <Assets assets={offerSummary.maker} catPresence={catPresence} />
          </CardContent>
        </Card>
      </div>

      {offerId && (
        <Card>
          <CardHeader className='pb-2'>
            <CardTitle className='text-lg font-medium flex items-center'>
              <Tags className='mr-2 h-5 w-5' />
              <Trans>Marketplaces</Trans>
            </CardTitle>
            <p className='text-sm text-muted-foreground'>
              <Trans>Share your offer on these marketplaces.</Trans>
            </p>
          </CardHeader>
          <CardContent className='flex flex-col gap-3'>
            <div className='flex flex-col md:flex-row items-start gap-8'>
              {marketplaces.map((marketplace) => (
                <MarketplaceCard
                  key={marketplace.id}
                  offer={offer || ''}
                  offerId={offerId}
                  offerSummary={offerSummary}
                  network={network || 'unknown'}
                  marketplace={marketplace}
                />
              ))}
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  );
}

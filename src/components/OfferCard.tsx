import { commands, NetworkKind, OfferRecord, OfferSummary } from '@/bindings';
import { NumberFormat } from '@/components/NumberFormat';
import { fromMojos, formatTimestamp } from '@/lib/utils';
import { useWalletState } from '@/state';
import { Trans } from '@lingui/react/macro';
import { t } from '@lingui/core/macro';
import {
  ShoppingBasketIcon,
  InfoIcon,
  Tags,
  HandCoinsIcon,
  Share,
  Copy,
} from 'lucide-react';
import { useEffect, useState } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from './ui/card';
import { cn } from '@/lib/utils';
import { Assets } from '@/components/Assets';
import { MarketplaceCard } from '@/components/MarketplaceCard';
import { marketplaces } from '@/lib/marketplaces';
import { shareText } from '@buildyourwebapp/tauri-plugin-sharesheet';
import { platform } from '@tauri-apps/plugin-os';
import { toast } from 'react-toastify';

// Interface to track CAT presence in wallet
interface CatPresence {
  [assetId: string]: boolean;
}

interface OfferCardProps {
  record?: OfferRecord;
  summary?: OfferSummary;
  content?: React.ReactNode;
  children?: React.ReactNode;
}

export function OfferCard({ record, summary, content }: OfferCardProps) {
  const walletState = useWalletState();
  // State to track which CATs are present in the wallet
  const [catPresence, setCatPresence] = useState<CatPresence>({});
  const [network, setNetwork] = useState<NetworkKind | null>(null);
  const isMobile = platform() === 'ios' || platform() === 'android';

  const offerSummary = summary || record?.summary;
  const offerId = record?.offer_id || '';
  const offer = record?.offer || undefined;

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

  // Check if CATs in the receiving section are present in the wallet
  useEffect(() => {
    if (!offerSummary) return;
    const checkCatPresence = async () => {
      const presence: CatPresence = {};

      // Check each CAT in the maker section (receiving)
      for (const assetId of Object.keys(offerSummary.maker.cats)) {
        try {
          const response = await commands.getCat({ asset_id: assetId });
          presence[assetId] = !!response.cat; // true if cat exists, false otherwise
        } catch (error) {
          console.error(`Error checking CAT presence for ${assetId}:`, error);
          presence[assetId] = false;
        }
      }

      setCatPresence(presence);
    };

    checkCatPresence();
  }, [offerSummary]);

  useEffect(() => {
    commands.getNetwork({}).then((data) => setNetwork(data.kind));
  }, []);

  const getStatusStyles = (status: string) => {
    switch (status.toLowerCase()) {
      case 'active':
        return 'text-green-600 dark:text-green-500';
      case 'completed':
        return 'text-blue-600 dark:text-blue-500';
      case 'cancelled':
        return 'text-amber-600 dark:text-amber-500';
      case 'expired':
        return 'text-red-600 dark:text-red-500';
      default:
        return '';
    }
  };

  const getStatusDotColor = (status: string) => {
    switch (status.toLowerCase()) {
      case 'active':
        return 'bg-green-500';
      case 'completed':
        return 'bg-blue-500';
      case 'cancelled':
        return 'bg-amber-500';
      case 'expired':
        return 'bg-red-500';
      default:
        return 'bg-gray-500';
    }
  };

  if (!offerSummary) return null;

  return (
    <div className='flex flex-col gap-4 max-w-screen-lg pr-1'>
      <Card>
        <CardHeader className='pb-2'>
          <div className='flex items-center justify-between'>
            <CardTitle className='text-lg font-medium flex items-center'>
              <InfoIcon className='mr-2 h-5 w-5' />
              <Trans>Offer Details</Trans>
            </CardTitle>
            {offer && (
              <div className='flex items-center gap-2'>
                {!isMobile && (
                  <button
                    onClick={handleCopy}
                    className='flex items-center gap-2 px-3 py-1.5 rounded-md border hover:bg-accent w-fit'
                    title={t`Copy offer`}
                  >
                    <Copy className='h-4 w-4' aria-hidden='true' />
                  </button>
                )}
                {isMobile && (
                  <button
                    onClick={handleShare}
                    className='flex items-center gap-2 px-3 py-1.5 rounded-md border hover:bg-accent w-fit'
                    title={t`Share offer`}
                  >
                    <Share className='h-4 w-4' aria-hidden='true' />
                  </button>
                )}
              </div>
            )}
          </div>
        </CardHeader>
        <CardContent className='flex flex-col gap-4'>
          <div className='grid grid-cols-1 md:grid-cols-3 gap-4'>
            {record?.status && (
              <div className='space-y-1.5'>
                <div className='text-sm font-medium text-muted-foreground'>
                  <Trans>Status</Trans>
                </div>
                <div className='flex items-center gap-2'>
                  <div
                    className={cn(
                      'w-2.5 h-2.5 rounded-full',
                      getStatusDotColor(record.status),
                    )}
                  />
                  <div
                    className={cn(
                      'font-medium',
                      getStatusStyles(record.status),
                    )}
                  >
                    {record.status === 'active'
                      ? 'Pending'
                      : record.status === 'completed'
                        ? 'Taken'
                        : record.status === 'cancelled'
                          ? 'Cancelled'
                          : 'Expired'}
                  </div>
                </div>
              </div>
            )}

            {record?.creation_timestamp && (
              <div className='space-y-1.5'>
                <div className='text-sm font-medium text-muted-foreground'>
                  <Trans>Created</Trans>
                </div>
                <div>
                  {new Date(record.creation_timestamp).toLocaleString()}
                </div>
              </div>
            )}

            <div className='space-y-1.5'>
              <div className='text-sm font-medium text-muted-foreground'>
                <Trans>Maker Fee</Trans>
              </div>
              <div>
                <NumberFormat
                  value={fromMojos(
                    offerSummary.fee,
                    walletState.sync.unit.decimals,
                  )}
                  minimumFractionDigits={0}
                  maximumFractionDigits={walletState.sync.unit.decimals}
                />{' '}
                {walletState.sync.unit.ticker}
              </div>
            </div>

            {(offerSummary.expiration_timestamp ||
              offerSummary.expiration_height) && (
              <div className='space-y-1.5'>
                <div className='text-sm font-medium text-muted-foreground'>
                  <Trans>Expires</Trans>
                </div>
                {offerSummary.expiration_timestamp && (
                  <div className='text-sm'>
                    {formatTimestamp(offerSummary.expiration_timestamp)}
                  </div>
                )}
                {offerSummary.expiration_height && (
                  <div className='text-sm text-muted-foreground'>
                    <Trans>Block:</Trans> {offerSummary.expiration_height}
                  </div>
                )}
              </div>
            )}
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
            <Assets
              assets={
                offerSummary.maker ?? {
                  xch: { amount: '0', royalty: '0' },
                  cats: {},
                  nfts: {},
                }
              }
              catPresence={catPresence}
            />
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

import { commands, NetworkKind, OfferRecord, OfferSummary } from '@/bindings';
import { NumberFormat } from '@/components/NumberFormat';
import { fromMojos, formatTimestamp } from '@/lib/utils';
import { useWalletState } from '@/state';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import {
  ArrowDownIcon,
  ArrowUpIcon,
  InfoIcon,
  ExternalLink,
  Tags,
} from 'lucide-react';
import { useEffect, useState } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from './ui/card';
import { cn } from '@/lib/utils';
import {
  dexieLink,
  uploadToDexie,
  uploadToMintGarden,
  offerIsOnDexie,
  offerIsOnMintGarden,
  isMintGardenSupportedForSummary,
} from '@/lib/offerUpload';
import { openUrl } from '@tauri-apps/plugin-opener';
import { toast } from 'react-toastify';
import { useErrors } from '@/hooks/useErrors';
import { Assets } from '@/components/Assets';
import StyledQRCode from '@/components/StyledQrCode';

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
  const { addError } = useErrors();
  // State to track which CATs are present in the wallet
  const [catPresence, setCatPresence] = useState<CatPresence>({});
  const [network, setNetwork] = useState<NetworkKind | null>(null);
  const [isOnDexie, setIsOnDexie] = useState<boolean | null>(null);
  const [isOnMintGarden, setIsOnMintGarden] = useState<boolean | null>(null);
  const [currentMintGardenLink, setCurrentMintGardenLink] =
    useState<string>('');

  const offerSummary = summary || record?.summary;
  const offerId = record?.offer_id || '';
  const offer = record?.offer || undefined;

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

  useEffect(() => {
    let isMounted = true;

    if (network !== 'unknown') {
      offerIsOnDexie(offerId, network === 'testnet').then((isOnDexie) => {
        if (isMounted) setIsOnDexie(isOnDexie);
      });

      offerIsOnMintGarden(offer || '', network === 'testnet').then(
        (isOnMintGarden) => {
          if (isMounted) setIsOnMintGarden(isOnMintGarden);
        },
      );
    }

    return () => {
      isMounted = false;
    };
  }, [network, offerId, offer]);

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

  const handleDexieAction = async () => {
    if (!offer) return;

    if (isOnDexie) {
      openUrl(dexieLink(offerId, network === 'testnet'));
    } else {
      const toastId = toast.loading(t`Uploading to Dexie...`);
      try {
        const url = await uploadToDexie(offer, network === 'testnet');
        toast.update(toastId, {
          render: t`Uploaded to Dexie!`,
          type: 'success',
          isLoading: false,
          autoClose: 3000,
        });
        setIsOnDexie(true);
      } catch (error: unknown) {
        toast.update(toastId, {
          render: t`Failed to upload to Dexie`,
          type: 'error',
          isLoading: false,
          autoClose: 3000,
        });
        addError({
          kind: 'upload',
          reason: `Failed to upload to Dexie: ${error instanceof Error ? error.message : String(error)}`,
        });
      }
    }
  };

  const handleMintGardenAction = async () => {
    if (!offer) return;

    if (isOnMintGarden) {
      openUrl(currentMintGardenLink);
    } else {
      const toastId = toast.loading(t`Uploading to MintGarden...`);
      try {
        const url = await uploadToMintGarden(offer, network === 'testnet');
        toast.update(toastId, {
          render: t`Uploaded to MintGarden!`,
          type: 'success',
          isLoading: false,
          autoClose: 3000,
        });
        setIsOnMintGarden(true);
        setCurrentMintGardenLink(url);
      } catch (error: unknown) {
        toast.update(toastId, {
          render: t`Failed to upload to MintGarden`,
          type: 'error',
          isLoading: false,
          autoClose: 3000,
        });
        addError({
          kind: 'upload',
          reason: `Failed to upload to MintGarden: ${error instanceof Error ? error.message : String(error)}`,
        });
      }
    }
  };

  if (!offerSummary) return null;

  return (
    <div className='flex flex-col gap-4 max-w-screen-lg'>
      <Card>
        <CardHeader className='pb-2'>
          <CardTitle className='text-lg font-medium flex items-center'>
            <InfoIcon className='mr-2 h-5 w-5' />
            <Trans>Offer Details</Trans>
          </CardTitle>
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

            {record?.creation_date && (
              <div className='space-y-1.5'>
                <div className='text-sm font-medium text-muted-foreground'>
                  <Trans>Created</Trans>
                </div>
                <div>{new Date(record.creation_date).toLocaleString()}</div>
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
              <ArrowUpIcon className='mr-2 h-5 w-5' />
              <Trans>Sending</Trans>
            </CardTitle>
            <p className='text-sm text-muted-foreground'>
              <Trans>The assets you have to pay to fulfill the offer.</Trans>
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
              <ArrowDownIcon className='mr-2 h-5 w-5' />
              <Trans>Receiving</Trans>
            </CardTitle>
            <p className='text-sm text-muted-foreground'>
              <Trans>The assets being given to you in the offer.</Trans>
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
            <div className='flex flex-col md:flex-row justify-start gap-8'>
              {/* Dexie Column */}
              <div className='flex flex-col items-center gap-4 w-full md:w-auto'>
                <button
                  onClick={handleDexieAction}
                  className='flex items-center gap-2 px-3 py-1.5 rounded-md border hover:bg-accent w-fit'
                >
                  <img
                    src='https://raw.githubusercontent.com/dexie-space/dexie-kit/refs/heads/main/svg/duck.svg'
                    className='h-4 w-4'
                    alt='Dexie logo'
                  />
                  <span className='text-sm'>
                    {isOnDexie ? (
                      <Trans>View on Dexie</Trans>
                    ) : (
                      <Trans>Upload to Dexie</Trans>
                    )}
                  </span>
                  {isOnDexie && <ExternalLink className='h-4 w-4' />}
                </button>

                {isOnDexie && (
                  <div className='bg-white p-2 rounded-lg'>
                    <StyledQRCode
                      data={dexieLink(offerId, network === 'testnet')}
                      width={200}
                      height={200}
                      cornersSquareOptions={{
                        type: 'extra-rounded',
                      }}
                      dotsOptions={{
                        type: 'rounded',
                        color: '#000000',
                      }}
                      backgroundOptions={{}}
                      image='https://raw.githubusercontent.com/dexie-space/dexie-kit/refs/heads/main/svg/duck.svg'
                      imageOptions={{
                        hideBackgroundDots: true,
                        imageSize: 0.4,
                        margin: 5,
                        saveAsBlob: true,
                      }}
                    />
                  </div>
                )}
              </div>

              {/* MintGarden Column */}
              {offerSummary &&
                isMintGardenSupportedForSummary(offerSummary) && (
                  <div className='flex flex-col items-center gap-4 w-full md:w-auto'>
                    <button
                      onClick={handleMintGardenAction}
                      className='flex items-center gap-2 px-3 py-1.5 rounded-md border hover:bg-accent w-fit'
                    >
                      <img
                        src='https://mintgarden.io/favicon.ico'
                        className='h-4 w-4'
                        alt='MintGarden logo'
                      />
                      <span className='text-sm'>
                        {isOnMintGarden ? (
                          <Trans>View on MintGarden</Trans>
                        ) : (
                          <Trans>Upload to MintGarden</Trans>
                        )}
                      </span>
                      {isOnMintGarden && <ExternalLink className='h-4 w-4' />}
                    </button>

                    {isOnMintGarden && (
                      <StyledQRCode
                        data={currentMintGardenLink}
                        width={200}
                        height={200}
                        cornersSquareOptions={{
                          type: 'extra-rounded',
                        }}
                        dotsOptions={{
                          type: 'rounded',
                          color: '#000000',
                        }}
                        backgroundOptions={{}}
                        imageOptions={{
                          hideBackgroundDots: false,
                          imageSize: 0.4,
                          margin: 5,
                          saveAsBlob: true,
                        }}
                      />
                    )}
                  </div>
                )}
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  );
}

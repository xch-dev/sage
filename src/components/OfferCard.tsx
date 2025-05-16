import {
  commands,
  OfferAssets,
  NetworkKind,
  OfferRecord,
  OfferSummary,
} from '@/bindings';
import { NumberFormat } from '@/components/NumberFormat';
import { nftUri } from '@/lib/nftUri';
import { fromMojos, formatTimestamp } from '@/lib/utils';
import { useWalletState } from '@/state';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import BigNumber from 'bignumber.js';
import {
  ArrowDownIcon,
  ArrowUpIcon,
  CheckCircleIcon,
  InfoIcon,
  XCircleIcon,
  QrCode,
  ExternalLink,
  Tags,
} from 'lucide-react';
import { useEffect, useState } from 'react';
import { CopyButton } from './CopyButton';
import { Badge } from './ui/badge';
import { Card, CardContent, CardHeader, CardTitle } from './ui/card';
import { Separator } from './ui/separator';
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from './ui/tooltip';
import { cn } from '@/lib/utils';
import { QRCodeDialog } from './QRCodeDialog';
import {
  dexieLink,
  uploadToDexie,
  uploadToMintGarden,
} from '@/lib/offerUpload';
import { openUrl } from '@tauri-apps/plugin-opener';
import { toast } from 'react-toastify';
import { useErrors } from '@/hooks/useErrors';

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
  const [qrOpen, setQrOpen] = useState(false);
  const [qrPlatform, setQrPlatform] = useState<'dexie' | 'mintgarden' | null>(
    null,
  );

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
    if (network !== 'unknown' && offerId) {
      // Check Dexie
      fetch(
        `https://${network === 'testnet' ? 'testnet.' : ''}api.dexie.space/v1/offers/${offerId}`,
      )
        .then((response) => response.json())
        .then((data) => setIsOnDexie(data.success === true))
        .catch(() => setIsOnDexie(false));

      // Check MintGarden
      fetch(
        `https://api.${network === 'testnet' ? 'testnet.' : ''}mintgarden.io/offers/${offerId}`,
      )
        .then((response) => response.json())
        .then((data) => {
          setIsOnMintGarden(data.success === true);
          if (data.success) {
            setCurrentMintGardenLink(
              `https://${network === 'testnet' ? 'testnet.' : ''}mintgarden.io/offers/${offerId}`,
            );
          }
        })
        .catch(() => setIsOnMintGarden(false));
    }
  }, [network, offerId]);

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
        openUrl(url);
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
        openUrl(url);
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
            <div className='flex flex-wrap gap-2'>
              <button
                onClick={handleDexieAction}
                className='flex items-center gap-2 px-3 py-1.5 rounded-md border hover:bg-accent'
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
                {isOnDexie && (
                  <>
                    <QrCode
                      className='h-4 w-4 cursor-pointer hover:text-primary'
                      onClick={(e) => {
                        e.stopPropagation();
                        setQrPlatform('dexie');
                        setQrOpen(true);
                      }}
                    />
                    <ExternalLink className='h-4 w-4' />
                  </>
                )}
              </button>

              <button
                onClick={handleMintGardenAction}
                className='flex items-center gap-2 px-3 py-1.5 rounded-md border hover:bg-accent'
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
                {isOnMintGarden && (
                  <>
                    <QrCode
                      className='h-4 w-4 cursor-pointer hover:text-primary'
                      onClick={(e) => {
                        e.stopPropagation();
                        setQrPlatform('mintgarden');
                        setQrOpen(true);
                      }}
                    />
                    <ExternalLink className='h-4 w-4' />
                  </>
                )}
              </button>
            </div>
          </CardContent>
        </Card>
      )}

      <QRCodeDialog
        isOpen={qrOpen}
        onClose={() => setQrOpen(false)}
        asset={null}
        qr_code_contents={
          qrPlatform === 'dexie'
            ? dexieLink(offerId, network === 'testnet')
            : currentMintGardenLink
        }
        title={
          qrPlatform === 'dexie' ? t`Dexie QR Code` : t`MintGarden QR Code`
        }
        description={
          qrPlatform === 'dexie'
            ? t`Scan this QR code to view the offer on dexie.space`
            : t`Scan this QR code to view the offer on mintgarden.io`
        }
      />
    </div>
  );
}

interface AssetsProps {
  assets: OfferAssets;
  catPresence?: CatPresence;
}

export function Assets({ assets, catPresence = {} }: AssetsProps) {
  const walletState = useWalletState();
  const amount = BigNumber(assets.xch.amount);

  if (
    amount.isLessThanOrEqualTo(0) &&
    Object.keys(assets.cats).length === 0 &&
    Object.keys(assets.nfts).length === 0
  ) {
    return <></>;
  }

  return (
    <div className='flex flex-col gap-3'>
      {amount.isGreaterThan(0) && (
        <div className='flex flex-col gap-2 rounded-lg border p-3'>
          <div className='flex items-center gap-2'>
            <Badge className='px-2 py-0.5'>
              <span className='truncate'>{walletState.sync.unit.ticker}</span>
            </Badge>

            <div className='font-medium'>
              <NumberFormat
                value={fromMojos(
                  BigNumber(amount).plus(assets.xch.royalty),
                  walletState.sync.unit.decimals,
                )}
                minimumFractionDigits={0}
                maximumFractionDigits={walletState.sync.unit.decimals}
              />{' '}
              {walletState.sync.unit.ticker}
            </div>
          </div>

          {BigNumber(assets.xch.royalty).isGreaterThan(0) && (
            <div className='text-sm text-muted-foreground'>
              <Trans>Amount includes</Trans>{' '}
              <NumberFormat
                value={fromMojos(
                  assets.xch.royalty,
                  walletState.sync.unit.decimals,
                )}
                minimumFractionDigits={0}
                maximumFractionDigits={walletState.sync.unit.decimals}
              />{' '}
              {walletState.sync.unit.ticker} <Trans>royalty</Trans>
            </div>
          )}
        </div>
      )}

      {Object.entries(assets.cats).map(([assetId, cat], i) => (
        <div key={i} className='flex flex-col gap-2 rounded-lg border p-3'>
          <div className='flex items-center gap-2'>
            <Badge className='px-2 py-0.5 bg-blue-600 text-white dark:bg-blue-600 dark:text-white'>
              <span className='truncate'>{cat.ticker ?? 'CAT'}</span>
            </Badge>

            <div className='font-medium'>
              <NumberFormat
                value={fromMojos(BigNumber(cat.amount).plus(cat.royalty), 3)}
                minimumFractionDigits={0}
                maximumFractionDigits={3}
              />{' '}
              {cat.name ?? cat.ticker ?? t`Unknown`}
            </div>

            {catPresence && assetId in catPresence && (
              <TooltipProvider>
                <Tooltip>
                  <TooltipTrigger>
                    {catPresence[assetId] ? (
                      <CheckCircleIcon className='h-5 w-5 text-green-500' />
                    ) : (
                      <XCircleIcon className='h-5 w-5 text-amber-500' />
                    )}
                  </TooltipTrigger>
                  <TooltipContent>
                    {catPresence[assetId] ? (
                      <p>
                        <Trans>This CAT is already in your wallet</Trans>
                      </p>
                    ) : (
                      <p>
                        <Trans>This CAT is not in your wallet yet</Trans>
                      </p>
                    )}
                  </TooltipContent>
                </Tooltip>
              </TooltipProvider>
            )}
          </div>

          <div className='flex items-center gap-2'>
            {cat.icon_url && (
              <img
                src={cat.icon_url}
                className='w-6 h-6 rounded-full'
                alt={t`CAT icon`}
              />
            )}

            <div className='text-sm font-mono text-muted-foreground'>
              {assetId.slice(0, 10) + '...' + assetId.slice(-10)}
            </div>

            <CopyButton value={assetId} className='w-4 h-4' />
          </div>

          {BigNumber(cat.royalty).isGreaterThan(0) && (
            <div className='text-sm text-muted-foreground'>
              <Trans>Amount includes</Trans>{' '}
              <NumberFormat
                value={fromMojos(cat.royalty, 3)}
                minimumFractionDigits={0}
                maximumFractionDigits={3}
              />{' '}
              {cat.ticker ?? 'CAT'} <Trans>royalty</Trans>
            </div>
          )}
        </div>
      ))}

      {Object.entries(assets.nfts).map(([launcherId, nft], i) => (
        <div key={i} className='flex flex-col gap-2 rounded-lg border p-3'>
          <div className='overflow-hidden flex items-center gap-2'>
            <div className='truncate flex items-center gap-2'>
              <Badge className='max-w-[100px] bg-green-600 text-white dark:bg-green-600 dark:text-white'>
                <span className='truncate'>
                  <Trans>NFT</Trans>
                </span>
              </Badge>
            </div>

            <div className='text-sm font-medium'>{nft.name ?? t`Unnamed`}</div>
          </div>

          <Separator className='my-1' />

          <div className='flex gap-1.5 items-center'>
            <img
              src={nftUri(nft.icon ? 'image/png' : null, nft.icon)}
              className='w-6 h-6 rounded-sm'
              alt={t`NFT preview`}
            />

            <div className='text-sm text-muted-foreground truncate font-mono'>
              {launcherId.slice(0, 10) + '...' + launcherId.slice(-10)}
            </div>

            <CopyButton value={launcherId} className='w-4 h-4' />
          </div>

          <Separator className='my-1' />

          <div className='flex gap-1.5 items-center text-sm text-muted-foreground truncate'>
            <span>
              <span className='text-neutral-600 dark:text-neutral-300'>
                {nft.royalty_ten_thousandths / 100}% {t`royalty to`}{' '}
              </span>
              <span className='font-mono'>
                {nft.royalty_address.slice(0, 10) +
                  '...' +
                  nft.royalty_address.slice(-10)}
              </span>
            </span>
            <CopyButton value={nft.royalty_address} className='w-4 h-4' />
          </div>
        </div>
      ))}
    </div>
  );
}

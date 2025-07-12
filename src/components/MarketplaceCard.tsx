import { OfferSummary } from '@/bindings';
import StyledQRCode from '@/components/StyledQrCode';
import { getOfferHash } from '@/lib/offerUpload';
import { OfferState } from '@/state';
import { shareText } from '@buildyourwebapp/tauri-plugin-sharesheet';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { openUrl } from '@tauri-apps/plugin-opener';
import { platform } from '@tauri-apps/plugin-os';
import { Copy, ExternalLink, Share } from 'lucide-react';
import { useEffect, useState } from 'react';
import { toast } from 'react-toastify';

export interface MarketplaceConfig {
  id: string;
  name: string;
  logo: string;
  qrCodeLogo: string | null;
  isSupported: (
    offer: OfferSummary | OfferState,
    isSplitting: boolean,
  ) => boolean;
  isOnMarketplace: (
    offer: string,
    offerId: string,
    isTestnet: boolean,
  ) => Promise<boolean>;
  uploadToMarketplace: (offer: string, isTestnet: boolean) => Promise<string>;
  getMarketplaceLink: (offerHash: string, isTestnet: boolean) => string;
}

interface MarketplaceCardProps {
  offer: string;
  offerId: string;
  offerSummary: OfferSummary | OfferState;
  network: 'mainnet' | 'testnet' | 'unknown';
  marketplace: MarketplaceConfig;
}

export function MarketplaceCard({
  offer,
  offerId,
  offerSummary,
  network,
  marketplace,
}: MarketplaceCardProps) {
  const [isOnMarketplace, setIsOnMarketplace] = useState<boolean | null>(null);
  const [offerHash, setOfferHash] = useState<string>('');
  const isMobile = platform() === 'ios' || platform() === 'android';

  useEffect(() => {
    let isMounted = true;

    if (network !== 'unknown' && marketplace.isSupported(offerSummary, false)) {
      getOfferHash(offer).then((hash) => {
        if (isMounted) setOfferHash(hash);
      });

      marketplace
        .isOnMarketplace(offer, offerId, network === 'testnet')
        .then((isOn) => {
          if (isMounted) setIsOnMarketplace(isOn);
        });
    }

    return () => {
      isMounted = false;
    };
  }, [network, offerId, offer, marketplace, offerSummary]);

  const handleMarketplaceAction = async () => {
    if (!offer) return;

    if (isOnMarketplace) {
      openUrl(marketplace.getMarketplaceLink(offerHash, network === 'testnet'));
    } else {
      const toastId = toast.loading(t`Uploading to ${marketplace.name}...`);
      try {
        await marketplace.uploadToMarketplace(offer, network === 'testnet');
        toast.update(toastId, {
          render: t`Uploaded to ${marketplace.name}!`,
          type: 'success',
          isLoading: false,
          autoClose: 3000,
        });
        setIsOnMarketplace(true);
      } catch (error: unknown) {
        toast.update(toastId, {
          render: `${error instanceof Error ? error.message : String(error)}`,
          type: 'error',
          isLoading: false,
          autoClose: 3000,
        });
      }
    }
  };

  const handleShare = async () => {
    if (!isOnMarketplace || !offerHash) return;

    try {
      const marketplaceLink = marketplace.getMarketplaceLink(
        offerHash,
        network === 'testnet',
      );
      await shareText(marketplaceLink, {
        title: t`${marketplace.name} link`,
        mimeType: 'text/uri-list',
      });
    } catch (error: unknown) {
      toast.error(`${error instanceof Error ? error.message : String(error)}`);
    }
  };

  const handleCopy = async () => {
    if (!isOnMarketplace || !offerHash) return;

    try {
      const marketplaceLink = marketplace.getMarketplaceLink(
        offerHash,
        network === 'testnet',
      );
      await navigator.clipboard.writeText(marketplaceLink);
      toast.success(t`Link copied to clipboard`);
    } catch (error: unknown) {
      toast.error(`${error instanceof Error ? error.message : String(error)}`);
    }
  };

  if (!marketplace.isSupported(offerSummary, false)) {
    return null;
  }

  return (
    <div className='flex flex-col items-center gap-4 w-auto'>
      <div className='flex items-center gap-2'>
        <button
          onClick={handleMarketplaceAction}
          className='flex items-center gap-2 px-3 py-1.5 rounded-md border hover:bg-accent w-fit'
          title={marketplace.getMarketplaceLink(
            offerHash,
            network === 'testnet',
          )}
        >
          <img
            src={marketplace.logo}
            className='h-4 w-4'
            alt={`${marketplace.name} logo`}
          />
          <span className='text-sm'>
            {isOnMarketplace ? (
              <Trans>View on {marketplace.name}</Trans>
            ) : (
              <Trans>Upload to {marketplace.name}</Trans>
            )}
          </span>
          {isOnMarketplace && <ExternalLink className='h-4 w-4' />}
        </button>

        {isOnMarketplace && isMobile && (
          <button
            onClick={handleShare}
            className='flex items-center gap-2 px-3 py-1.5 rounded-md border hover:bg-accent w-fit'
            title={t`Share marketplace link`}
          >
            <Share className='h-4 w-4' aria-hidden='true' />
          </button>
        )}

        {isOnMarketplace && !isMobile && (
          <button
            onClick={handleCopy}
            className='flex items-center gap-2 px-3 py-1.5 rounded-md border hover:bg-accent w-fit'
            title={t`Copy marketplace link`}
          >
            <Copy className='h-4 w-4' aria-hidden='true' />
          </button>
        )}
      </div>

      {isOnMarketplace && (
        <StyledQRCode
          data={marketplace.getMarketplaceLink(
            offerHash,
            network === 'testnet',
          )}
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
            hideBackgroundDots: true,
            imageSize: 0.4,
            margin: 5,
            saveAsBlob: true,
          }}
          {...(marketplace.qrCodeLogo && { image: marketplace.qrCodeLogo })}
        />
      )}
    </div>
  );
}

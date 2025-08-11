import { TokenRecord } from '@/bindings';
import { getAssetDisplayName } from '@/lib/utils';
import { useWalletState } from '@/state';
import { Trans } from '@lingui/react/macro';
import {
  Clock,
  FilePenLine,
  HandCoins,
  Handshake,
  UserRoundPlus,
} from 'lucide-react';
import { AssetIcon } from '../AssetIcon';
import { NumberFormat } from '../NumberFormat';
import { ConfirmationAlert } from './ConfirmationAlert';
import { ConfirmationCard } from './ConfirmationCard';

interface MintOptionConfirmationProps {
  underlyingAsset?: TokenRecord | null;
  underlyingAmount?: string;
  strikeAsset?: TokenRecord | null;
  strikeAmount?: string;
  expirationSeconds?: number;
}

export function MintOptionConfirmation({
  underlyingAsset,
  underlyingAmount,
  strikeAsset,
  strikeAmount,
  expirationSeconds,
}: MintOptionConfirmationProps) {
  const walletState = useWalletState();

  const formatExpiration = (seconds: number) => {
    const now = Math.floor(Date.now() / 1000);
    const diff = seconds - now;
    const days = Math.floor(diff / (24 * 60 * 60));
    const hours = Math.floor((diff % (24 * 60 * 60)) / (60 * 60));
    const minutes = Math.floor((diff % (60 * 60)) / 60);

    if (days > 0) return `${days}d ${hours}h ${minutes}m`;
    if (hours > 0) return `${hours}h ${minutes}m`;
    return `${minutes}m`;
  };

  return (
    <div className='space-y-4 text-xs'>
      <ConfirmationAlert
        icon={UserRoundPlus}
        title={<Trans>Option Minting</Trans>}
        variant='warning'
      >
        <Trans>
          You are minting a new option contract. This will lock up the
          underlying funds until the expiration, or until the current owner
          chooses to exercise it.
        </Trans>
      </ConfirmationAlert>

      {underlyingAsset && underlyingAmount && (
        <ConfirmationCard
          icon={<HandCoins className='h-8 w-8 text-blue-500' />}
          title={<Trans>Underlying Asset (Locked)</Trans>}
        >
          <div className='flex items-center gap-3 rounded-lg'>
            <AssetIcon
              asset={{
                icon_url: underlyingAsset.icon_url,
                kind: 'token',
                revocation_address: underlyingAsset.revocation_address,
              }}
              size='md'
              className='flex-shrink-0'
            />
            <div className='flex flex-col'>
              <div className='font-medium'>
                <NumberFormat
                  value={underlyingAmount}
                  minimumFractionDigits={0}
                  maximumFractionDigits={underlyingAsset.precision}
                />{' '}
                {getAssetDisplayName(
                  underlyingAsset.name,
                  underlyingAsset.ticker,
                  'token',
                )}
              </div>
              <div className='text-xs text-muted-foreground'>
                {underlyingAsset.asset_id
                  ? `${underlyingAsset.asset_id.slice(0, 8)}...${underlyingAsset.asset_id.slice(-8)}`
                  : walletState.sync.unit.ticker}
              </div>
            </div>
          </div>
        </ConfirmationCard>
      )}

      {strikeAsset && strikeAmount && (
        <ConfirmationCard
          icon={<Handshake className='h-8 w-8 text-green-500' />}
          title={<Trans>Strike Price</Trans>}
        >
          <div className='flex items-center gap-3 rounded-lg'>
            <AssetIcon
              asset={{
                icon_url: strikeAsset.icon_url,
                kind: 'token',
                revocation_address: strikeAsset.revocation_address,
              }}
              size='md'
              className='flex-shrink-0'
            />
            <div className='flex flex-col'>
              <div className='font-medium'>
                <NumberFormat
                  value={strikeAmount}
                  minimumFractionDigits={0}
                  maximumFractionDigits={strikeAsset.precision}
                />{' '}
                {getAssetDisplayName(
                  strikeAsset.name,
                  strikeAsset.ticker,
                  'token',
                )}
              </div>
              <div className='text-xs text-muted-foreground'>
                {strikeAsset.asset_id
                  ? `${strikeAsset.asset_id.slice(0, 8)}...${strikeAsset.asset_id.slice(-8)}`
                  : walletState.sync.unit.ticker}
              </div>
            </div>
          </div>
        </ConfirmationCard>
      )}

      {expirationSeconds && (
        <ConfirmationCard
          icon={<Clock className='h-8 w-8 text-orange-500' />}
          title={<Trans>Expiration</Trans>}
        >
          <div className='text-muted-foreground'>
            <Trans>
              This option expires in {formatExpiration(expirationSeconds)}
            </Trans>
          </div>
        </ConfirmationCard>
      )}

      <ConfirmationCard
        icon={<FilePenLine className='h-8 w-8 text-purple-500' />}
        title={<Trans>Option Contract</Trans>}
      >
        <div className='text-muted-foreground'>
          <Trans>This option contract will be created on the blockchain.</Trans>
        </div>
      </ConfirmationCard>
    </div>
  );
}

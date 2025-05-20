import { OfferAssets } from '@/bindings';
import { NumberFormat } from '@/components/NumberFormat';
import { nftUri } from '@/lib/nftUri';
import { fromMojos } from '@/lib/utils';
import { useWalletState } from '@/state';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import BigNumber from 'bignumber.js';
import { CheckCircleIcon, XCircleIcon } from 'lucide-react';
import { CopyButton } from './CopyButton';
import { Badge } from './ui/badge';
import { Separator } from './ui/separator';
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from './ui/tooltip';

// Interface to track CAT presence in wallet
export interface CatPresence {
  [assetId: string]: boolean;
}

export interface AssetsProps {
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
              <span className='break-words'>
                {cat.name ?? cat.ticker ?? t`Unknown`}
              </span>
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

            <div className='text-sm font-mono text-muted-foreground truncate'>
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
              <span className='truncate'>{cat.ticker ?? 'CAT'}</span>{' '}
              <Trans>royalty</Trans>
            </div>
          )}
        </div>
      ))}

      {Object.entries(assets.nfts).map(([launcherId, nft], i) => (
        <div key={i} className='flex flex-col gap-2 rounded-lg border p-3'>
          <div className='overflow-hidden flex items-center gap-2'>
            <div className='truncate flex items-center gap-2'>
              <Badge className='max-w-[100px] bg-green-600 text-white dark:bg-green-600 dark:text-white'>
                <Trans>NFT</Trans>
              </Badge>
            </div>

            <div className='text-sm font-medium break-words'>
              {nft.name ?? t`Unnamed`}
            </div>
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

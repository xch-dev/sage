import { OfferAsset } from '@/bindings';
import { NumberFormat } from '@/components/NumberFormat';
import { fromMojos } from '@/lib/utils';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import BigNumber from 'bignumber.js';
import { CheckCircleIcon, XCircleIcon } from 'lucide-react';
import { AssetIcon } from './AssetIcon';
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
export type CatPresence = Record<string, boolean>;

export interface AssetsProps {
  assets: OfferAsset[];
  catPresence?: CatPresence;
}

export function Assets({ assets, catPresence = {} }: AssetsProps) {
  return (
    <div className='flex flex-col gap-3'>
      {assets.map(({ asset, amount, royalty, nft_royalty }, i) => (
        <div key={i} className='flex flex-col gap-2 rounded-lg border p-3'>
          <div className='flex items-center gap-2'>
            <Badge className='px-2 py-0.5 bg-blue-600 text-white dark:bg-blue-600 dark:text-white'>
              <span className='truncate'>
                {asset.ticker ??
                  (asset.kind === 'token' ? 'CAT' : asset.kind.toUpperCase())}
              </span>
            </Badge>

            <div className='font-medium'>
              {asset.kind !== 'nft' && (
                <NumberFormat
                  value={fromMojos(
                    BigNumber(amount).plus(royalty),
                    asset.precision,
                  )}
                  maximumFractionDigits={asset.precision}
                />
              )}{' '}
              <span className='text-sm font-medium break-words'>
                {asset.name ?? asset.ticker ?? t`Unknown`}
              </span>
            </div>
            {catPresence &&
              asset.kind === 'token' &&
              asset.asset_id &&
              asset.asset_id in catPresence && (
                <TooltipProvider>
                  <Tooltip>
                    <TooltipTrigger>
                      {catPresence[asset.asset_id] ? (
                        <CheckCircleIcon className='h-5 w-5 text-green-500' />
                      ) : (
                        <XCircleIcon className='h-5 w-5 text-amber-500' />
                      )}
                    </TooltipTrigger>
                    <TooltipContent>
                      {catPresence[asset.asset_id] ? (
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
            <AssetIcon iconUrl={asset.icon_url} kind={asset.kind} />
            {asset.asset_id && (
              <>
                <div className='text-sm font-mono text-muted-foreground truncate'>
                  {asset.asset_id.slice(0, 10) +
                    '...' +
                    asset.asset_id.slice(-10)}
                </div>

                <CopyButton value={asset.asset_id} className='w-4 h-4' />
              </>
            )}
          </div>

          {BigNumber(royalty).isGreaterThan(0) && (
            <div className='text-sm text-muted-foreground'>
              <Trans>Amount includes</Trans>{' '}
              <NumberFormat
                value={fromMojos(royalty, asset.precision)}
                maximumFractionDigits={asset.precision}
              />{' '}
              <span className='truncate'>{asset.ticker ?? 'CAT'}</span>{' '}
              <Trans>royalty</Trans>
            </div>
          )}

          {nft_royalty && (
            <>
              <Separator className='my-1' />

              <div className='flex gap-1.5 items-center text-sm text-muted-foreground truncate'>
                <span>
                  <span className='text-neutral-600 dark:text-neutral-300'>
                    {nft_royalty.royalty_basis_points / 100}% {t`royalty to`}{' '}
                  </span>
                  <span className='font-mono'>
                    {nft_royalty.royalty_address.slice(0, 10) +
                      '...' +
                      nft_royalty.royalty_address.slice(-10)}
                  </span>
                </span>
                <CopyButton
                  value={nft_royalty.royalty_address}
                  className='w-4 h-4'
                />
              </div>
            </>
          )}
        </div>
      ))}
    </div>
  );
}

import { OfferAsset, OfferRecord } from '@/bindings';
import { NumberFormat } from '@/components/NumberFormat';
import { formatTimestamp, fromMojos } from '@/lib/utils';
import { t } from '@lingui/core/macro';
import BigNumber from 'bignumber.js';
import { AssetIcon } from './AssetIcon';

export interface OfferSummaryCardProps {
  record: OfferRecord;
  content: React.ReactNode;
}

export function OfferSummaryCard({ record, content }: OfferSummaryCardProps) {
  return (
    <div className='block p-4 rounded-sm bg-neutral-100 dark:bg-neutral-900'>
      <div className='flex justify-between'>
        <div className='grid grid-cols-1 md:grid-cols-3 gap-4'>
          <div className='flex flex-col gap-1'>
            <div>
              {record.status === 'active'
                ? 'Pending'
                : record.status === 'completed'
                  ? 'Taken'
                  : record.status === 'cancelled'
                    ? 'Cancelled'
                    : 'Expired'}
            </div>
            <div className='text-muted-foreground text-sm'>
              {new Date(record.creation_timestamp * 1000).toLocaleString()}
            </div>
            {record.summary?.expiration_timestamp && (
              <div className='text-muted-foreground text-sm'>
                <span>
                  Expires:{' '}
                  {formatTimestamp(
                    record.summary.expiration_timestamp,
                    'short',
                    'medium',
                  )}
                </span>
              </div>
            )}
            {record.summary?.expiration_height && (
              <div className='text-muted-foreground text-sm'>
                <span>Block: {record.summary.expiration_height}</span>
              </div>
            )}
          </div>

          <AssetPreview label='Offered' assets={record.summary.maker} />
          <AssetPreview label='Requested' assets={record.summary.taker} />
        </div>

        {content}
      </div>
    </div>
  );
}

interface AssetPreviewProps {
  label: string;
  assets: OfferAsset[];
}

function AssetPreview({ label, assets }: AssetPreviewProps) {
  return (
    <div className='flex flex-col gap-1 w-[125px] lg:w-[200px] xl:w-[300px]'>
      <div>{label}</div>
      {assets.map(({ amount, royalty, asset }) => (
        <div className='flex items-center gap-2' key={asset.asset_id ?? 'xch'}>
          <AssetIcon asset={asset} size='md' />
          <div className='text-sm text-muted-foreground truncate'>
            {asset.kind !== 'nft' && asset.kind !== 'option' && (
              <NumberFormat
                value={fromMojos(
                  BigNumber(amount).plus(royalty),
                  asset.precision,
                )}
                minimumFractionDigits={0}
                maximumFractionDigits={asset.precision}
              />
            )}{' '}
            {asset.name ?? asset.ticker ?? t`Unknown`}
          </div>
        </div>
      ))}
    </div>
  );
}

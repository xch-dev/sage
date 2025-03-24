import { OfferAssets, OfferRecord } from '@/bindings';
import { NumberFormat } from '@/components/NumberFormat';
import { nftUri } from '@/lib/nftUri';
import { fromMojos, formatTimestamp } from '@/lib/utils';
import { useWalletState } from '@/state';
import { t } from '@lingui/core/macro';
import BigNumber from 'bignumber.js';

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
              {new Date(record.creation_date).toLocaleString()}
            </div>
            {record.summary?.expiration_timestamp && (
              <div className='text-muted-foreground text-sm'>
                <span>
                  Expires:{' '}
                  {formatTimestamp(record.summary.expiration_timestamp)}
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
  assets: OfferAssets;
}

function AssetPreview({ label, assets }: AssetPreviewProps) {
  const walletState = useWalletState();

  return (
    <div className='flex flex-col gap-1 w-[125px] lg:w-[200px] xl:w-[300px]'>
      <div>{label}</div>
      {BigNumber(assets.xch.amount)
        .plus(assets.xch.royalty)
        .isGreaterThan(0) && (
        <div className='flex items-center gap-2'>
          <img
            alt={t`XCH`}
            src='https://icons.dexie.space/xch.webp'
            className='w-8 h-8'
          />

          <div className='text-sm text-muted-foreground truncate'>
            <NumberFormat
              value={fromMojos(
                BigNumber(assets.xch.amount).plus(assets.xch.royalty),
                walletState.sync.unit.decimals,
              )}
              minimumFractionDigits={0}
              maximumFractionDigits={walletState.sync.unit.decimals}
            />{' '}
            {walletState.sync.unit.ticker}
          </div>
        </div>
      )}
      {Object.entries(assets.cats).map(([, cat], i) => (
        <div className='flex items-center gap-2' key={i}>
          <img
            alt={cat.name ?? cat.ticker ?? t`Unknown`}
            src={cat.icon_url!}
            className='w-8 h-8'
          />

          <div className='text-sm text-muted-foreground truncate'>
            <NumberFormat
              value={fromMojos(BigNumber(cat.amount).plus(cat.royalty), 3)}
              minimumFractionDigits={0}
              maximumFractionDigits={3}
            />{' '}
            {cat.name ?? cat.ticker ?? t`Unknown`}
          </div>
        </div>
      ))}
      {Object.entries(assets.nfts).map(([, nft], i) => (
        <div className='flex items-center gap-2' key={i}>
          <img
            alt={nft.name ?? t`Unknown`}
            src={nftUri(nft.icon ? 'image/png' : null, nft.icon)}
            className='w-8 h-8'
          />

          <div className='text-sm text-muted-foreground truncate'>
            {nft.name ?? t`Unknown`}
          </div>
        </div>
      ))}
    </div>
  );
}

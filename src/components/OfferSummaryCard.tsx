import { OfferAssets, OfferRecord } from '@/bindings';
import { nftUri } from '@/lib/nftUri';
import { fromMojos } from '@/lib/utils';
import { useWalletState } from '@/state';
import BigNumber from 'bignumber.js';
import { t } from '@lingui/core/macro';
import { NumberFormat } from '@/components/NumberFormat';

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
              {record.creation_date}
            </div>
            {record.summary?.expiration_date && (
              <div className='text-muted-foreground text-sm'>
                <span>Expires: {record.summary.expiration_date}</span>
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
      {Object.entries(assets.cats).map(([_assetId, cat]) => (
        <div className='flex items-center gap-2'>
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
      {Object.entries(assets.nfts).map(([_nftId, nft]) => (
        <div className='flex items-center gap-2'>
          <img
            alt={nft.name ?? t`Unknown`}
            src={nftUri(nft.image_mime_type, nft.image_data)}
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

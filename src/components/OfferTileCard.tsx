import { Asset, OfferAsset, OfferRecord } from '@/bindings';
import { formatCompactNumber, formatNumber, formatRelativeTime } from '@/i18n';
import { formatTimestamp, fromMojos } from '@/lib/utils';
import { t } from '@lingui/core/macro';
import BigNumber from 'bignumber.js';
import { ArrowDownUp, Blocks, Clock3, Hourglass } from 'lucide-react';
import { AssetIcon } from './AssetIcon';
import { OfferStatusBadge, offerStatusLabel } from './OfferStatusBadge';
import { SelectionState } from './SelectableCard';
import { Checkbox } from './ui/checkbox';

const MAX_STACKED_ICONS = 3;

export interface OfferTileCardProps {
  record: OfferRecord;
  content: React.ReactNode;
  selectionState?: SelectionState;
  /** Reference time in ms for relative times; defaults to the current time. */
  now?: number;
}

/** Square, compact offer card: offered assets over requested assets. */
export function OfferTileCard({
  record,
  content,
  selectionState = null,
  now,
}: OfferTileCardProps) {
  const { maker, taker, expiration_timestamp, expiration_height } =
    record.summary;

  return (
    <div className='flex aspect-square flex-col gap-2 rounded-sm border border-border bg-card p-3'>
      <div className='flex items-center gap-2'>
        {selectionState !== null && (
          <Checkbox
            checked={selectionState[0]}
            className='flex-shrink-0'
            aria-label={selectionState[0] ? t`Deselect offer` : t`Select offer`}
          />
        )}
        <OfferStatusBadge status={record.status} />
        <div className='ml-auto'>{content}</div>
      </div>

      <div className='flex min-h-0 flex-1 flex-col justify-center gap-1'>
        <TileSide assets={maker} label={t`Offered`} />
        <ArrowDownUp
          className='h-4 w-4 self-center text-muted-foreground'
          aria-hidden='true'
        />
        <TileSide assets={taker} label={t`Requested`} />
      </div>

      <div className='flex items-center justify-between gap-2 text-xs text-muted-foreground'>
        <TileExpiry
          timestamp={expiration_timestamp}
          height={expiration_height}
          now={now}
        />
        <span
          className='flex items-center gap-1'
          title={formatTimestamp(record.creation_timestamp, 'short', 'medium')}
        >
          <Clock3 className='h-3 w-3' aria-hidden='true' />
          {formatRelativeTime(record.creation_timestamp, now)}
        </span>
      </div>
    </div>
  );
}

function hasAmount(asset: Asset): boolean {
  return asset.kind !== 'nft' && asset.kind !== 'option';
}

function assetAmount({ amount, royalty, asset }: OfferAsset): BigNumber {
  return fromMojos(BigNumber(amount).plus(royalty), asset.precision);
}

function assetName({ asset }: OfferAsset): string {
  const name = hasAmount(asset)
    ? (asset.ticker ?? asset.name)
    : (asset.name ?? asset.ticker);
  return name ?? t`Unknown`;
}

interface TileSideProps {
  assets: OfferAsset[];
  label: string;
}

function TileSide({ assets, label }: TileSideProps) {
  if (assets.length === 0) return null;

  const [first] = assets;
  const extra = assets.length - 1;

  return (
    <div className='flex min-w-0 items-center gap-2' title={label}>
      <div className='flex flex-shrink-0 -space-x-3'>
        {assets.slice(0, MAX_STACKED_ICONS).map((item) => (
          <div
            key={item.asset.asset_id ?? 'xch'}
            className='flex h-9 w-9 items-center justify-center overflow-hidden rounded-full border-2 border-card bg-muted'
          >
            <AssetIcon asset={item.asset} size='lg' />
          </div>
        ))}
      </div>
      <div className='flex min-w-0 flex-col leading-tight'>
        {hasAmount(first.asset) && (
          <span className='truncate text-sm font-medium'>
            {formatCompactNumber(assetAmount(first))}
          </span>
        )}
        <span className='truncate text-xs text-muted-foreground'>
          {assetName(first)}
        </span>
      </div>
      {extra > 0 && (
        <span className='flex-shrink-0 text-xs text-muted-foreground'>
          +{extra}
        </span>
      )}
    </div>
  );
}

interface TileExpiryProps {
  timestamp: number | null;
  height: number | null;
  now?: number;
}

function TileExpiry({ timestamp, height, now }: TileExpiryProps) {
  if (timestamp) {
    return (
      <span
        className='flex items-center gap-1'
        title={formatTimestamp(timestamp, 'short', 'medium')}
      >
        <Hourglass className='h-3 w-3' aria-hidden='true' />
        {formatRelativeTime(timestamp, now)}
      </span>
    );
  }

  if (height) {
    return (
      <span
        className='flex items-center gap-1'
        title={t`Expires at block ${height}`}
      >
        <Blocks className='h-3 w-3' aria-hidden='true' />
        {height}
      </span>
    );
  }

  // Keeps the age pinned to the right edge.
  return <span />;
}

function describeAssets(assets: OfferAsset[]): string {
  return assets
    .map((item) =>
      hasAmount(item.asset)
        ? `${formatNumber({
            value: assetAmount(item),
            maximumFractionDigits: item.asset.precision,
          })} ${assetName(item)}`
        : assetName(item),
    )
    .join(', ');
}

/** Spoken summary of a tile, which otherwise conveys status and times with icons. */
export function offerSummaryLabel(record: OfferRecord, now?: number): string {
  const status = offerStatusLabel(record.status);
  const offeredAssets = describeAssets(record.summary.maker);
  const requestedAssets = describeAssets(record.summary.taker);
  const summary = t`${status} offer: ${offeredAssets} for ${requestedAssets}`;

  const expiration = record.summary.expiration_timestamp;
  if (!expiration) return summary;

  const expiry = formatRelativeTime(expiration, now);
  return t`${summary}, expires ${expiry}`;
}

import { Asset, OfferAsset, OfferRecord } from '@/bindings';
import { formatCompactNumber, formatNumber, formatRelativeTime } from '@/i18n';
import { formatTimestamp, fromMojos } from '@/lib/utils';
import { t } from '@lingui/core/macro';
import BigNumber from 'bignumber.js';
import { Blocks, Clock3, Hourglass } from 'lucide-react';
import { AssetIcon } from './AssetIcon';
import { OfferStatusBadge, offerStatusLabel } from './OfferStatusBadge';
import { SelectionState } from './SelectableCard';
import { Checkbox } from './ui/checkbox';

const DUST_THRESHOLD = 0.001;

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
  const { maker, taker } = record.summary;

  return (
    <div className='flex h-full flex-col gap-2 rounded-lg border border-border bg-card p-3 shadow'>
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

      <div className='flex flex-1 flex-col justify-center gap-2'>
        <TileSide assets={maker} label={t`Offered`} />
        <div className='border-t border-border' aria-hidden='true' />
        <TileSide assets={taker} label={t`Requested`} />
      </div>

      <div className='text-xs text-muted-foreground'>
        <TileTime record={record} now={now} />
      </div>
    </div>
  );
}

function isLive(record: OfferRecord): boolean {
  return record.status === 'active' || record.status === 'pending';
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
      <div className='relative flex-shrink-0'>
        <div className='flex h-8 w-8 items-center justify-center overflow-hidden rounded-full bg-muted'>
          <AssetIcon asset={first.asset} size='lg' />
        </div>
        {extra > 0 && (
          <span className='absolute -bottom-1 -right-2 rounded-full border border-card bg-secondary px-1 text-[10px] font-medium leading-4 text-secondary-foreground'>
            +{extra}
          </span>
        )}
      </div>
      <div className='flex min-w-0 flex-col leading-tight'>
        {hasAmount(first.asset) && <TileAmount item={first} />}
        <span className='truncate text-xs text-muted-foreground'>
          {assetName(first)}
        </span>
      </div>
    </div>
  );
}

function TileAmount({ item }: { item: OfferAsset }) {
  const amount = assetAmount(item);
  const isDust = !amount.isZero() && amount.abs().lt(DUST_THRESHOLD);
  const text = isDust
    ? `<${formatNumber({ value: DUST_THRESHOLD })}`
    : formatCompactNumber(amount);
  const exact = formatNumber({
    value: amount,
    maximumFractionDigits: item.asset.precision,
  });

  return (
    <span className='truncate text-sm font-medium' title={exact}>
      {text}
    </span>
  );
}

interface TileTimeProps {
  record: OfferRecord;
  now?: number;
}

/** Expiry for live offers, otherwise the offer's age. */
function TileTime({ record, now }: TileTimeProps) {
  const { expiration_timestamp, expiration_height } = record.summary;
  if (isLive(record) && (expiration_timestamp || expiration_height)) {
    return (
      <TileExpiry
        timestamp={expiration_timestamp}
        height={expiration_height}
        now={now}
      />
    );
  }

  return (
    <span
      className='flex items-center gap-1'
      title={formatTimestamp(record.creation_timestamp, 'short', 'medium')}
    >
      <Clock3 className='h-3 w-3' aria-hidden='true' />
      {formatRelativeTime(record.creation_timestamp, now)}
    </span>
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

  return null;
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
  if (!expiration || !isLive(record)) return summary;

  const expiry = formatRelativeTime(expiration, now);
  return t`${summary}, expires ${expiry}`;
}

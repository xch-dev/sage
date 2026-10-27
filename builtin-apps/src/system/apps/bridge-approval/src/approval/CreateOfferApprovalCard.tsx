import { Handshake } from 'lucide-react';
import type { RustBridgeApprovalRequest } from 'sage-system-app-sdk';
import { ApprovalDetailRow, ApprovalMetaPill } from './shared';

interface Props {
  approval: Extract<RustBridgeApprovalRequest, { kind: 'createOffer' }>;
  appName: string;
}

type OfferAsset = Props['approval']['summary']['offered'][number];

const KIND_LABELS: Record<OfferAsset['kind'], string> = {
  xch: 'XCH',
  cat: 'CAT',
  nft: 'NFT',
  option: 'Option',
};

function assetName(asset: OfferAsset) {
  if (asset.kind === 'xch') return asset.ticker ?? 'XCH';
  if (asset.kind === 'cat') return asset.ticker ?? asset.name ?? 'Unknown CAT';
  return asset.name ?? `Unknown ${KIND_LABELS[asset.kind]}`;
}

function formatAmount(amount: string, precision: number) {
  if (precision === 0) return amount;

  const digits = amount.padStart(precision + 1, '0');
  const whole = digits.slice(0, -precision);
  const fraction = digits.slice(-precision).replace(/0+$/, '');

  return fraction ? `${whole}.${fraction}` : whole;
}

function assetAmount(asset: OfferAsset) {
  const amount = formatAmount(asset.amount, asset.precision);
  if (asset.kind === 'nft' || asset.kind === 'option') return amount;

  // An asset posing as XCH gets no unit, so its amount cannot read as "1 XCH".
  const unit =
    asset.kind === 'xch'
      ? (asset.ticker ?? 'XCH')
      : asset.impersonatesXch
        ? null
        : asset.ticker;
  const formatted = unit ? `${amount} ${unit}` : amount;

  return asset.precisionKnown
    ? formatted
    : `${formatted} (precision unknown, assuming ${asset.precision} decimals)`;
}

function formatExpiry(seconds: number) {
  return new Date(seconds * 1000).toLocaleString();
}

function AssetList({ title, assets }: { title: string; assets: OfferAsset[] }) {
  return (
    <div className='space-y-2 rounded-xl border bg-background/70 p-3'>
      <div className='text-xs font-semibold uppercase tracking-wide text-muted-foreground'>
        {title}
      </div>

      {assets.length === 0 ? (
        <div className='text-sm text-muted-foreground'>Nothing</div>
      ) : (
        assets.map((asset, index) => (
          <div
            key={`${asset.assetId ?? 'xch'}-${index}`}
            className='space-y-1 rounded-md border px-2 py-2'
          >
            <div className='flex flex-wrap items-center gap-2'>
              <ApprovalMetaPill>{KIND_LABELS[asset.kind]}</ApprovalMetaPill>
              <div className='min-w-0 break-all text-sm font-medium'>
                {assetName(asset)}
              </div>
            </div>
            {asset.impersonatesXch ? (
              <div className='rounded-lg border border-destructive/40 bg-destructive/10 p-2 text-xs text-destructive'>
                This is not XCH. It is {asset.kind === 'option' ? 'an' : 'a'}{' '}
                {KIND_LABELS[asset.kind]} whose name or ticker is XCH. Check the
                {asset.kind === 'cat'
                  ? ' asset'
                  : ` ${KIND_LABELS[asset.kind]}`}{' '}
                ID before approving.
              </div>
            ) : null}
            <ApprovalDetailRow label='Amount' value={assetAmount(asset)} />
            {asset.kind === 'xch' || asset.kind === 'cat' ? (
              <ApprovalDetailRow
                label='Mojos'
                value={asset.amount}
                mono
                breakAll
              />
            ) : null}
            {asset.assetId ? (
              <ApprovalDetailRow
                label={
                  asset.kind === 'cat'
                    ? 'Asset ID'
                    : `${KIND_LABELS[asset.kind]} ID`
                }
                value={asset.assetId}
                mono
                breakAll
              />
            ) : null}
          </div>
        ))
      )}
    </div>
  );
}

export function CreateOfferApprovalCard({ approval, appName }: Props) {
  const { summary } = approval;

  return (
    <div className='space-y-3'>
      <div className='flex items-start gap-3'>
        <div className='rounded-xl border bg-background p-2 text-muted-foreground'>
          <Handshake className='h-4 w-4' />
        </div>

        <div className='min-w-0 flex-1'>
          <div className='flex flex-wrap items-center gap-2'>
            <div className='text-sm font-medium'>Create offer</div>
            <ApprovalMetaPill>Wallet</ApprovalMetaPill>
          </div>

          <div className='mt-1 text-xs text-muted-foreground'>
            {appName} wants your wallet to create and sign an offer. Coins for
            the offered assets are reserved until the offer is taken or
            cancelled, and the offer is listed on the Offers page.
          </div>
        </div>
      </div>

      <AssetList title='You offer' assets={summary.offered} />
      <AssetList title='You request' assets={summary.requested} />

      <div className='space-y-2 rounded-xl border bg-background/70 p-3'>
        <ApprovalDetailRow
          label='Fee'
          value={`${formatAmount(summary.fee, 12)} XCH`}
        />
        <ApprovalDetailRow
          label='Expires'
          value={
            summary.expiresAtSecond !== null
              ? formatExpiry(summary.expiresAtSecond)
              : 'Never expires'
          }
        />
      </div>
    </div>
  );
}

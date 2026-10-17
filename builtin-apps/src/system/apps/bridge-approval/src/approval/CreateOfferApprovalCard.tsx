import { Handshake } from 'lucide-react';
import type { RustBridgeApprovalRequest } from 'sage-system-app-sdk';
import { ApprovalDetailRow, ApprovalMetaPill } from './shared';

interface Props {
  approval: Extract<RustBridgeApprovalRequest, { kind: 'createOffer' }>;
  appName: string;
}

type OfferAsset = Props['approval']['summary']['offered'][number];

function assetLabel(asset: OfferAsset) {
  if (!asset.assetId) return asset.ticker ?? 'XCH';
  return asset.ticker ?? asset.name ?? 'Unknown asset';
}

function formatAmount(amount: string, precision: number | null) {
  if (precision === null || precision === 0) return amount;

  const digits = amount.padStart(precision + 1, '0');
  const whole = digits.slice(0, -precision);
  const fraction = digits.slice(-precision).replace(/0+$/, '');

  return fraction ? `${whole}.${fraction}` : whole;
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
            <ApprovalDetailRow
              label={assetLabel(asset)}
              value={formatAmount(asset.amount, asset.precision)}
            />
            <ApprovalDetailRow
              label='Raw amount'
              value={asset.amount}
              mono
              breakAll
            />
            {asset.assetId ? (
              <ApprovalDetailRow
                label='Asset ID'
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
        {summary.expiresAtSecond !== null ? (
          <ApprovalDetailRow
            label='Expires'
            value={formatExpiry(summary.expiresAtSecond)}
          />
        ) : null}
      </div>
    </div>
  );
}

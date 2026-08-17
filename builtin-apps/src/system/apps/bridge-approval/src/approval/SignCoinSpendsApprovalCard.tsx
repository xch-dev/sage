import { FileSignature } from 'lucide-react';
import type { RustBridgeApprovalRequest } from '@sage-system-app/sdk';
import { ApprovalDetailRow, ApprovalMetaPill } from './shared';

interface Props {
  approval: Extract<RustBridgeApprovalRequest, { kind: 'signCoinSpends' }>;
  appName: string;
}

function assetLabel(
  asset: Props['approval']['summary']['inputs'][number]['asset'],
) {
  if (!asset) return 'XCH';
  return asset.ticker ?? asset.name ?? asset.assetId ?? asset.kind.toUpperCase();
}

export function SignCoinSpendsApprovalCard({ approval, appName }: Props) {
  const { summary } = approval;

  return (
    <div className='space-y-3'>
      <div className='flex items-start gap-3'>
        <div className='rounded-xl border bg-background p-2 text-muted-foreground'>
          <FileSignature className='h-4 w-4' />
        </div>

        <div className='min-w-0 flex-1'>
          <div className='flex flex-wrap items-center gap-2'>
            <div className='text-sm font-medium'>Sign coin spends</div>
            <ApprovalMetaPill>Wallet</ApprovalMetaPill>
            {approval.partialSign ? (
              <ApprovalMetaPill>Partial signature</ApprovalMetaPill>
            ) : null}
          </div>

          <div className='mt-1 text-xs text-muted-foreground'>
            {appName} wants your wallet to sign a custom transaction. Review
            every input and output before approving.
          </div>
        </div>
      </div>

      <div className='space-y-2 rounded-xl border bg-background/70 p-3'>
        <ApprovalDetailRow label='Inputs' value={summary.inputs.length} />
        <ApprovalDetailRow label='Fee' value={String(summary.fee)} />
      </div>

      <div className='space-y-3'>
        {summary.inputs.map((input, inputIndex) => (
          <div
            key={input.coinId}
            className='space-y-2 rounded-xl border bg-background/70 p-3'
          >
            <div className='text-xs font-semibold uppercase tracking-wide text-muted-foreground'>
              Input {inputIndex + 1} · {assetLabel(input.asset)}
            </div>
            <ApprovalDetailRow label='Amount' value={String(input.amount)} />
            <ApprovalDetailRow
              label='From'
              value={input.address}
              mono
              breakAll
            />
            <ApprovalDetailRow
              label='Coin ID'
              value={input.coinId}
              mono
              breakAll
            />

            <div className='space-y-2 border-t pt-2'>
              {input.outputs.map((output, outputIndex) => (
                <div
                  key={output.coinId}
                  className='space-y-1 rounded-md border px-2 py-2'
                >
                  <div className='text-[11px] font-medium uppercase tracking-wide text-muted-foreground'>
                    Output {outputIndex + 1}
                    {output.burning
                      ? ' · Burn'
                      : output.receiving
                        ? ' · Receiving'
                        : ''}
                  </div>
                  <ApprovalDetailRow
                    label='Amount'
                    value={String(output.amount)}
                  />
                  <ApprovalDetailRow
                    label='To'
                    value={output.address}
                    mono
                    breakAll
                  />
                </div>
              ))}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

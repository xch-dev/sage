import { Wallet } from 'lucide-react';
import type { RustBridgeApprovalRequest } from 'sage-system-app-sdk';
import { ApprovalDetailRow, ApprovalMetaPill } from './shared';

interface Props {
  approval: Extract<RustBridgeApprovalRequest, { kind: 'sendXch' }>;
  appName: string;
  expanded: boolean;
}

function truncateMiddle(value: string, maxLength = 120) {
  if (value.length <= maxLength) {
    return value;
  }

  const head = Math.ceil((maxLength - 1) / 2);
  const tail = Math.floor((maxLength - 1) / 2);
  return `${value.slice(0, head)}…${value.slice(value.length - tail)}`;
}

function memoKey(memo: string, indexWithinSameValue: number) {
  return `${memo}::${indexWithinSameValue}`;
}

export function SendXchApprovalCard({ approval, appName, expanded }: Props) {
  const summary = approval.summary;

  const hasFee = summary.fee !== '0';
  const memos = summary.memos ?? [];
  const hasMemos = memos.length > 0;

  const memoEntries = memos.map(
    (memo: string, index: number, all: string[]) => {
      const duplicateIndex = all
        .slice(0, index)
        .filter((previous: string) => previous === memo).length;

      return {
        key: memoKey(memo, duplicateIndex),
        value: memo,
      };
    },
  );

  return (
    <div className='space-y-3'>
      <div className='flex items-start gap-3'>
        <div className='rounded-xl border bg-background p-2 text-muted-foreground'>
          <Wallet className='h-4 w-4' />
        </div>

        <div className='min-w-0 flex-1'>
          <div className='flex flex-wrap items-center gap-2'>
            <div className='text-sm font-medium'>Send XCH</div>
            <ApprovalMetaPill>Wallet</ApprovalMetaPill>
          </div>

          <div className='mt-1 text-xs text-muted-foreground'>
            {appName} wants to send funds from your wallet.
          </div>
        </div>
      </div>

      <div className='space-y-2 rounded-xl border bg-background/70 p-3'>
        <ApprovalDetailRow label='Amount' value={summary.amount} />
        <ApprovalDetailRow label='To' value={summary.address} mono breakAll />
        {hasFee ? <ApprovalDetailRow label='Fee' value={summary.fee} /> : null}
        {hasMemos ? (
          <ApprovalDetailRow label='Memos' value={`${memos.length} attached`} />
        ) : null}
      </div>

      {expanded && hasMemos ? (
        <div className='rounded-xl border bg-background/70 p-3'>
          <div className='mb-2 text-xs font-medium text-muted-foreground'>
            Memo previews
          </div>

          <div className='space-y-2'>
            {memoEntries.map((memo, index) => (
              <div
                key={memo.key}
                className='rounded-md border px-2 py-2 text-xs'
              >
                <div className='mb-1 text-[11px] uppercase tracking-wide text-muted-foreground'>
                  Memo {index + 1}
                </div>
                <div className='break-all font-mono'>
                  {truncateMiddle(memo.value, 160)}
                </div>
              </div>
            ))}
          </div>
        </div>
      ) : null}
    </div>
  );
}

import { Wallet } from 'lucide-react';
import type { RustBridgeApprovalRequest } from 'sage-system-app-sdk';
import { parseMojos, parseXchFee } from './fee';
import { ApprovalDetailRow, ApprovalMetaPill } from './shared';

interface Props {
  approval: Extract<RustBridgeApprovalRequest, { kind: 'sendXch' }>;
  appName: string;
  expanded: boolean;
  working: boolean;
  feeInput: string;
  onFeeInputChange: (value: string) => void;
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

export function SendXchApprovalCard({
  approval,
  appName,
  expanded,
  working,
  feeInput,
  onFeeInputChange,
}: Props) {
  const summary = approval.summary;

  const selectedFee = parseXchFee(feeInput);
  const suggestedFee = parseMojos(summary.fee);
  const hasSuggestedFee = suggestedFee !== null && suggestedFee.mojos !== '0';
  const isUsingSuggestedFee =
    hasSuggestedFee && selectedFee?.mojos === suggestedFee.mojos;
  const hasInvalidSuggestion =
    summary.fee.trim().length > 0 && suggestedFee === null;
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
        {hasMemos ? (
          <ApprovalDetailRow label='Memos' value={`${memos.length} attached`} />
        ) : null}
      </div>

      <div className='space-y-3 px-1 pt-1'>
        <div>
          <label
            htmlFor='send-xch-fee'
            className='text-xs font-medium text-muted-foreground'
          >
            Network fee
          </label>
          <div className='fee-input-frame mt-1 flex items-center rounded-md border bg-background/50 px-3 transition-colors'>
            <input
              id='send-xch-fee'
              type='text'
              inputMode='decimal'
              autoComplete='off'
              spellCheck={false}
              disabled={working}
              value={feeInput}
              onChange={(event) => onFeeInputChange(event.target.value)}
              aria-invalid={selectedFee === null}
              aria-describedby='send-xch-fee-description'
              className='min-w-0 flex-1 bg-transparent py-2 text-sm font-mono outline-none disabled:cursor-not-allowed disabled:opacity-60'
            />
            <span className='ml-2 text-xs text-muted-foreground'>XCH</span>
          </div>

          <div
            id='send-xch-fee-description'
            className={`mt-1 text-xs ${selectedFee ? 'text-muted-foreground' : 'text-destructive'}`}
          >
            {selectedFee
              ? `${selectedFee.mojos} mojos`
              : 'Enter a non-negative fee with no more than 12 decimal places.'}
          </div>
        </div>

        {hasSuggestedFee && !isUsingSuggestedFee ? (
          <div className='flex justify-center'>
            <button
              type='button'
              disabled={working}
              onClick={() => onFeeInputChange(suggestedFee.xch)}
              className='inline-flex items-center justify-center rounded-md border border-border bg-transparent px-3 py-1.5 text-xs font-medium transition-colors hover:border-primary/50 hover:bg-muted disabled:cursor-not-allowed disabled:opacity-60'
            >
              Use suggested fee {suggestedFee.xch} XCH
            </button>
          </div>
        ) : null}

        {hasInvalidSuggestion ? (
          <div className='rounded-lg border border-destructive/40 bg-destructive/10 p-2 text-xs text-destructive'>
            {appName} suggested an invalid fee. It will not be used.
          </div>
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

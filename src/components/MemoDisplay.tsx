import { formatMemo, Memo, MemoMode } from '@/types/CoinMemo.ts';

interface MemoDisplayProps {
  memo: Memo;
}

export function MemoDisplay({ memo }: MemoDisplayProps) {
  const isHex = memo.mode === MemoMode.Hex;
  const label = isHex ? 'Hex' : 'Text';
  const value = formatMemo(memo);

  return (
    <div className='inline-flex items-center gap-2'>
      <span className='rounded-md border border-border bg-muted px-2 py-0.5 text-xs font-medium uppercase tracking-wide text-muted-foreground'>
        {label}
      </span>

      <span className='font-mono text-sm text-foreground'>{value}</span>
    </div>
  );
}

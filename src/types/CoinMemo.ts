import { withHexPrefix } from '@/lib/utils.ts';

export enum MemoMode {
  Text = 'text',
  Hex = 'hex',
}

export interface Memo {
  mode: MemoMode;
  value: string;
}

export function formatMemo(memo: Memo): string {
  return memo.mode === MemoMode.Hex ? withHexPrefix(memo.value) : memo.value;
}


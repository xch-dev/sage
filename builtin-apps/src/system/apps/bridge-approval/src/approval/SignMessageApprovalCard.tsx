import { MessageSquareText } from 'lucide-react';
import type { RustBridgeApprovalRequest } from '@sage-system-app/sdk';
import { ApprovalDetailRow, ApprovalMetaPill } from './shared';

interface Props {
  approval: Extract<RustBridgeApprovalRequest, { kind: 'signMessage' }>;
  appName: string;
}

function decodeHexMessage(message: string) {
  if (!/^(?:[0-9a-fA-F]{2})+$/.test(message)) return null;

  const bytes = new Uint8Array(
    message.match(/.{2}/g)?.map((byte) => Number.parseInt(byte, 16)) ?? [],
  );
  const decoded = new TextDecoder('utf-8', { fatal: true }).decode(bytes);

  return /^[\t\n\r\x20-\x7e]*$/.test(decoded) ? decoded : null;
}

export function SignMessageApprovalCard({ approval, appName }: Props) {
  let decodedMessage: string | null = null;

  try {
    decodedMessage = decodeHexMessage(approval.message);
  } catch {
    decodedMessage = null;
  }

  return (
    <div className='space-y-3'>
      <div className='flex items-start gap-3'>
        <div className='rounded-xl border bg-background p-2 text-muted-foreground'>
          <MessageSquareText className='h-4 w-4' />
        </div>

        <div className='min-w-0 flex-1'>
          <div className='flex flex-wrap items-center gap-2'>
            <div className='text-sm font-medium'>Sign message</div>
            <ApprovalMetaPill>Wallet</ApprovalMetaPill>
          </div>

          <div className='mt-1 text-xs text-muted-foreground'>
            {appName} wants to sign this message with a key from your active
            wallet.
          </div>
        </div>
      </div>

      <div className='space-y-2 rounded-xl border bg-background/70 p-3'>
        <ApprovalDetailRow
          label='Public key'
          value={approval.publicKey}
          mono
          breakAll
        />
        <ApprovalDetailRow
          label='Message'
          value={approval.message}
          mono
          breakAll
        />
      </div>

      {decodedMessage !== null ? (
        <div className='rounded-xl border bg-background/70 p-3'>
          <div className='mb-2 text-xs font-medium uppercase tracking-wide text-muted-foreground'>
            Decoded UTF-8 preview
          </div>
          <div className='whitespace-pre-wrap break-words text-sm'>
            {decodedMessage}
          </div>
        </div>
      ) : null}
    </div>
  );
}

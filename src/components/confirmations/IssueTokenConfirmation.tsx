import { CopyBox } from '@/components/CopyBox';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { CoinsIcon } from 'lucide-react';
import { toast } from 'react-toastify';

interface IssueTokenConfirmationProps {
  name: string;
  ticker: string;
  amount: string;
}

export function IssueTokenConfirmation({
  name,
  ticker,
  amount,
}: IssueTokenConfirmationProps) {
  return (
    <div className='space-y-3 text-xs'>
      <div className='p-2 bg-blue-50 dark:bg-blue-950 border border-blue-200 dark:border-blue-800 rounded-md text-blue-800 dark:text-blue-300'>
        <div className='font-medium mb-1 flex items-center'>
          <CoinsIcon className='h-3 w-3 mr-1' />
          <Trans>Token Issuance</Trans>
        </div>
        <div>
          <Trans>
            You are issuing a new token. This will create a CAT (Chia Asset
            Token) that can be sent to other users and traded on exchanges.
          </Trans>
        </div>
      </div>

      <div className='flex items-start gap-3 border border-neutral-200 dark:border-neutral-800 rounded-md p-3'>
        <div className='overflow-hidden rounded-md flex-shrink-0 w-16 h-16 border border-neutral-200 dark:border-neutral-800 flex items-center justify-center bg-neutral-50 dark:bg-neutral-900'>
          <CoinsIcon className='h-8 w-8 text-blue-500' />
        </div>
        <div className='break-words whitespace-pre-wrap flex-1'>
          <div className='font-medium mb-2'>{name}</div>

          <div className='grid grid-cols-2 gap-2'>
            <div>
              <div className='text-muted-foreground text-xs mb-1'>
                <Trans>Ticker</Trans>
              </div>
              <div className='font-medium'>{ticker}</div>
            </div>

            <div>
              <div className='text-muted-foreground text-xs mb-1'>
                <Trans>Amount</Trans>
              </div>
              <div className='font-medium'>
                {amount} {ticker}
              </div>
            </div>
          </div>
        </div>
      </div>

      <div className='text-muted-foreground'>
        <Trans>
          Once issued, this token will appear in your wallet. You can then send
          it to other addresses or create offers to trade it.
        </Trans>
      </div>
    </div>
  );
}

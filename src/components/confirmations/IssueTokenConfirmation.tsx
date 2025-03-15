import { Trans } from '@lingui/react/macro';
import { CoinsIcon } from 'lucide-react';
import { ConfirmationAlert } from './ConfirmationAlert';
import { ConfirmationCard } from './ConfirmationCard';

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
      <ConfirmationAlert
        icon={CoinsIcon}
        title={<Trans>Token Issuance</Trans>}
        variant='info'
      >
        <Trans>
          You are issuing a new token. This will create a CAT (Chia Asset Token)
          that can be sent to other users and traded on exchanges.
        </Trans>
      </ConfirmationAlert>

      <ConfirmationCard
        icon={<CoinsIcon className='h-8 w-8 text-blue-500' />}
        title={name}
      >
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
      </ConfirmationCard>

      <div className='text-muted-foreground'>
        <Trans>
          Once issued, this token will appear in your wallet. You can then send
          it to other addresses or create offers to trade it.
        </Trans>
      </div>
    </div>
  );
}

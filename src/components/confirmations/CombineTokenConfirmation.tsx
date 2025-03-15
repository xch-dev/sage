import { CoinRecord } from '@/bindings';
import { fromMojos } from '@/lib/utils';
import { Trans } from '@lingui/react/macro';
import { MergeIcon } from 'lucide-react';
import { ConfirmationAlert } from './ConfirmationAlert';
import { ConfirmationCard } from './ConfirmationCard';

interface CombineTokenConfirmationProps {
  coins: CoinRecord[];
  ticker: string;
  precision: number;
}

export function CombineTokenConfirmation({
  coins,
  ticker,
  precision,
}: CombineTokenConfirmationProps) {
  const totalAmount = coins.reduce(
    (acc, coin) => acc + BigInt(coin.amount),
    BigInt(0),
  );

  // Convert to string to avoid BigNumber rendering issues
  const formattedTotalAmount = String(
    fromMojos(totalAmount.toString(), precision),
  );

  return (
    <div className='space-y-3 text-xs'>
      <ConfirmationAlert
        icon={MergeIcon}
        title={<Trans>Combine Coins</Trans>}
        variant='info'
      >
        <Trans>
          You are combining multiple coins into a single coin. This can help
          reduce the number of coins in your wallet.
        </Trans>
      </ConfirmationAlert>

      <ConfirmationCard title={<Trans>Coins to Combine</Trans>}>
        <div className='space-y-2 max-h-40 overflow-y-auto'>
          {coins.map((coin) => (
            <div
              key={coin.coin_id}
              className='flex justify-between items-center border-b border-neutral-100 dark:border-neutral-800 pb-1 last:border-0 last:pb-0'
            >
              <div className='truncate flex-1 mr-2'>
                {coin.coin_id.substring(0, 8)}...
                {coin.coin_id.substring(coin.coin_id.length - 8)}
              </div>
              <div className='font-medium'>
                {String(fromMojos(coin.amount.toString(), precision))} {ticker}
              </div>
            </div>
          ))}
        </div>
      </ConfirmationCard>

      <ConfirmationCard>
        <div className='grid grid-cols-2 gap-4'>
          <div>
            <div className='text-muted-foreground mb-1'>
              <Trans>Total Amount</Trans>
            </div>
            <div className='font-medium'>
              {formattedTotalAmount} {ticker}
            </div>
          </div>
          <div>
            <div className='text-muted-foreground mb-1'>
              <Trans>Result</Trans>
            </div>
            <div className='font-medium'>
              <Trans>1 combined coin</Trans>
            </div>
          </div>
        </div>
      </ConfirmationCard>
    </div>
  );
}

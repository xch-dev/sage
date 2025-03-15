import { CoinRecord } from '@/bindings';
import { CopyBox } from '@/components/CopyBox';
import { fromMojos } from '@/lib/utils';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { SplitIcon } from 'lucide-react';
import { toast } from 'react-toastify';

interface SplitTokenConfirmationProps {
  coins: CoinRecord[];
  outputCount: number;
  ticker: string;
  precision: number;
}

export function SplitTokenConfirmation({
  coins,
  outputCount,
  ticker,
  precision,
}: SplitTokenConfirmationProps) {
  const totalAmount = coins.reduce(
    (sum, coin) => sum + BigInt(coin.amount),
    BigInt(0),
  );

  // Convert to string to avoid BigNumber rendering issues
  const formattedTotalAmount = String(
    fromMojos(totalAmount.toString(), precision),
  );
  const approximateOutputAmount = (
    Number(formattedTotalAmount) / outputCount
  ).toFixed(precision);

  return (
    <div className='space-y-3 text-xs'>
      <div className='p-2 bg-blue-50 dark:bg-blue-950 border border-blue-200 dark:border-blue-800 rounded-md text-blue-800 dark:text-blue-300'>
        <div className='font-medium mb-1 flex items-center'>
          <SplitIcon className='h-3 w-3 mr-1' />
          <Trans>Split Details</Trans>
        </div>
        <div>
          <Trans>
            You are splitting {coins.length}{' '}
            {coins.length === 1 ? 'coin' : 'coins'} into {outputCount} new
            coins. Each new coin will have approximately{' '}
            {approximateOutputAmount} {ticker}.
          </Trans>
        </div>
      </div>

      <div className='border border-neutral-200 dark:border-neutral-800 rounded-md p-3'>
        <div className='font-medium mb-2'>
          <Trans>Coins to Split</Trans>
        </div>
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
      </div>

      <div className='border border-neutral-200 dark:border-neutral-800 rounded-md p-3'>
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
              <Trans>Output Count</Trans>
            </div>
            <div className='font-medium'>{outputCount} coins</div>
          </div>
        </div>
      </div>
    </div>
  );
}

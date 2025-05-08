import { Amount, AssetCoinType } from '@/bindings';
import { fromMojos } from '@/lib/utils';
import { useWalletState } from '@/state';
import BigNumber from 'bignumber.js';
import { NumberFormat } from './NumberFormat';

interface AmountCellProps {
  amount: Amount;
  type: AssetCoinType;
}

export function AmountCell({ amount, type }: AmountCellProps) {
  const walletState = useWalletState();
  const amountNum = BigNumber(amount);
  const decimals = type === 'cat' ? 3 : walletState.sync.unit.decimals;

  return (
    <div className='whitespace-nowrap'>
      <span
        className={
          amountNum.eq(0)
            ? 'text-blue-600'
            : amountNum.gt(0)
              ? 'text-green-600'
              : 'text-red-600'
        }
        aria-label={`${fromMojos(amount, decimals)} ${type.toUpperCase()}`}
      >
        {type === 'nft' || type === 'did' ? (
          amountNum.eq(0) ? (
            'Edited'
          ) : amountNum.gt(0) ? (
            'Received'
          ) : (
            'Sent'
          )
        ) : (
          <NumberFormat
            value={fromMojos(amount, decimals)}
            minimumFractionDigits={0}
            maximumFractionDigits={decimals}
          />
        )}
      </span>
    </div>
  );
}

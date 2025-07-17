import { Amount, AssetKind } from '@/bindings';
import { fromMojos } from '@/lib/utils';
import BigNumber from 'bignumber.js';
import { NumberFormat } from './NumberFormat';
import { t } from '@lingui/core/macro';

interface AmountCellProps {
  amount: Amount;
  assetKind: AssetKind;
  precision: number;
}

export function AmountCell({ amount, assetKind, precision }: AmountCellProps) {
  const amountNum = BigNumber(amount);

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
        aria-label={`${fromMojos(amount, precision)} ${assetKind}`}
      >
        {assetKind === 'nft' || assetKind === 'did' ? (
          amountNum.eq(0) ? (
            t`Edited`
          ) : amountNum.gt(0) ? (
            t`Received`
          ) : (
            t`Sent`
          )
        ) : (
          <NumberFormat
            value={fromMojos(amount, precision)}
            maximumFractionDigits={precision}
          />
        )}
      </span>
    </div>
  );
}

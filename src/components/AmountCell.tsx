import { Amount, AssetKind } from '@/bindings';
import { fromMojos } from '@/lib/utils';
import { Trans } from '@lingui/react/macro';
import BigNumber from 'bignumber.js';
import { NumberFormat } from './NumberFormat';

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
      >
        {assetKind === 'nft' ||
        assetKind === 'did' ||
        assetKind === 'option' ? (
          amountNum.eq(0) ? (
            <Trans>Edited</Trans>
          ) : amountNum.gt(0) ? (
            <Trans>Received</Trans>
          ) : (
            <Trans>Sent</Trans>
          )
        ) : (
          <NumberFormat
            value={fromMojos(amount, precision)}
            minimumFractionDigits={0}
            maximumFractionDigits={precision}
          />
        )}
      </span>
    </div>
  );
}

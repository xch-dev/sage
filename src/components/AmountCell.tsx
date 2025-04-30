import { fromMojos } from '@/lib/utils';
import { useWalletState } from '@/state';
import { NumberFormat } from './NumberFormat';

interface AmountCellProps {
  amount: string;
  type: string;
}

export function AmountCell({ amount, type }: AmountCellProps) {
  const walletState = useWalletState();
  const isPositive = amount.startsWith('+');
  const typeLower = type.toLowerCase();
  const decimals = typeLower === 'cat' ? 3 : walletState.sync.unit.decimals;
  const sign =
    amount === 'edited' ? 'edited' : isPositive ? 'received' : 'sent';

  return (
    <div className='whitespace-nowrap'>
      <span
        className={
          amount === 'edited'
            ? 'text-blue-600'
            : isPositive
              ? 'text-green-600'
              : 'text-red-600'
        }
        aria-label={`${sign} ${typeLower === 'nft' || typeLower === 'did' || amount === 'edited' ? '' : fromMojos(amount, decimals)} ${typeLower === 'xch' ? 'XCH' : typeLower === 'cat' ? 'CAT' : type}`}
      >
        {typeLower === 'nft' || typeLower === 'did' || amount === 'edited' ? (
          <span>{sign}</span>
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

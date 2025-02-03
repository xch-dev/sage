import { useWalletState } from '@/state';
import { fromMojos } from '@/lib/utils';
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
  const sign = isPositive ? 'received' : 'sent';

  return (
    <div className='text-right whitespace-nowrap'>
      <span
        className={isPositive ? 'text-green-600' : 'text-red-600'}
        aria-label={`${sign} ${fromMojos(amount, decimals)} ${typeLower === 'xch' ? 'XCH' : typeLower === 'cat' ? 'CAT' : type}`}
      >
        <NumberFormat
          value={fromMojos(amount, decimals)}
          minimumFractionDigits={0}
          maximumFractionDigits={decimals}
        />
      </span>
    </div>
  );
}

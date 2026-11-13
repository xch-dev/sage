import { Amount, Unit } from '@/bindings';
import { formatNumber } from '@/i18n';
import { fromMojos } from '@/lib/utils';
import { Trans } from '@lingui/react/macro';
import BigNumber from 'bignumber.js';
import { AlertCircleIcon } from 'lucide-react';
import { Alert, AlertDescription, AlertTitle } from './ui/alert';

export interface UndeclaredFeeAlertProps {
  fee: Amount;
  declaredFee: Amount | undefined;
  unit: Unit;
  className?: string;
}

/**
 * Warns when a transaction's fee includes value it leaves over without
 * declaring it as a fee, which a dApp-supplied spend can do to hide a fee.
 */
export function UndeclaredFeeAlert({
  fee,
  declaredFee,
  unit,
  className,
}: UndeclaredFeeAlertProps) {
  const actual = BigNumber(fee || 0);

  if (actual.isEqualTo(BigNumber(declaredFee || 0))) {
    return null;
  }

  const formattedFee = formatNumber({
    value: fromMojos(actual, unit.precision),
    minimumFractionDigits: 0,
    maximumFractionDigits: unit.precision,
  });
  const ticker = unit.ticker;

  return (
    <Alert variant='warning' className={className}>
      <AlertCircleIcon className='h-4 w-4' aria-hidden='true' />
      <AlertTitle>
        <Trans>Undeclared Fee</Trans>
      </AlertTitle>
      <AlertDescription>
        <Trans>
          This transaction pays a fee of {formattedFee} {ticker}, which differs
          from the fee it declares.
        </Trans>
      </AlertDescription>
    </Alert>
  );
}

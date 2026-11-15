import { Unit, VaultSpendRecord } from '@/bindings';
import { formatNumber } from '@/i18n';
import { fromMojos } from '@/lib/utils';
import { Trans } from '@lingui/react/macro';
import BigNumber from 'bignumber.js';
import { AlertCircleIcon, ShieldCheckIcon } from 'lucide-react';
import { Alert, AlertDescription, AlertTitle } from './ui/alert';

export interface VaultSpendAlertProps {
  vault: VaultSpendRecord | null | undefined;
  unit: Unit;
  className?: string;
}

/** Whether Sage will sign the vault's spend, which it refuses if clear signing
 *  can't verify it, or if it changes the vault's custody or melts it. */
export function isVaultSpendSignable(
  vault: VaultSpendRecord | null | undefined,
): boolean {
  return !vault || (vault.verified && !vault.custody_changed && !vault.melted);
}

/**
 * What the open vault's spend does, as checked with clear signing before its
 * custody signs the transaction.
 */
export function VaultSpendAlert({
  vault,
  unit,
  className,
}: VaultSpendAlertProps) {
  if (!vault) return null;

  if (!vault.verified) {
    const error = vault.error ?? '';

    return (
      <Alert variant='destructive' className={className}>
        <AlertCircleIcon className='h-4 w-4' aria-hidden='true' />
        <AlertTitle>
          <Trans>Vault spend can&apos;t be verified</Trans>
        </AlertTitle>
        <AlertDescription>
          <Trans>
            Sage couldn&apos;t check what this transaction does with your vault,
            so it won&apos;t sign it: {error}
          </Trans>
        </AlertDescription>
      </Alert>
    );
  }

  if (vault.custody_changed || vault.melted) {
    return (
      <Alert variant='destructive' className={className}>
        <AlertCircleIcon className='h-4 w-4' aria-hidden='true' />
        <AlertTitle>
          {vault.melted ? (
            <Trans>This transaction melts your vault</Trans>
          ) : (
            <Trans>This transaction changes your vault&apos;s keys</Trans>
          )}
        </AlertTitle>
        <AlertDescription>
          <Trans>
            Whoever made it could take control of the vault, so Sage won&apos;t
            sign it.
          </Trans>
        </AlertDescription>
      </Alert>
    );
  }

  const unaccounted = BigNumber(vault.unaccounted_xch || 0);
  const formattedUnaccounted = formatNumber({
    value: fromMojos(unaccounted, unit.precision),
    minimumFractionDigits: 0,
    maximumFractionDigits: unit.precision,
  });
  const ticker = unit.ticker;

  return (
    <Alert
      variant={unaccounted.isZero() ? 'default' : 'warning'}
      className={className}
    >
      <ShieldCheckIcon className='h-4 w-4' aria-hidden='true' />
      <AlertTitle>
        <Trans>Signed by your vault</Trans>
      </AlertTitle>
      <AlertDescription>
        {unaccounted.isZero() ? (
          <Trans>
            Clear signing checked this transaction. Your vault keeps the same
            keys.
          </Trans>
        ) : (
          <Trans>
            Your vault keeps the same keys, but {formattedUnaccounted} {ticker}{' '}
            leaves it without being accounted for, which is left as an extra
            fee.
          </Trans>
        )}
      </AlertDescription>
    </Alert>
  );
}

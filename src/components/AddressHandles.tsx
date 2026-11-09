import { commands, Error as BackendError, HandleRecord } from '@/bindings';
import { CopyButton } from '@/components/CopyButton';
import { Button } from '@/components/ui/button';
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from '@/components/ui/popover';
import { useAddressHandles } from '@/hooks/useAddressHandles';
import { refreshAddressLabels } from '@/lib/addressContacts';
import { HandleTrust, handleTrust } from '@/lib/addressHandles';
import { handleErrorMessage } from '@/lib/handles';
import { cn, formatTimestamp } from '@/lib/utils';
import { plural, t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import {
  AlertTriangleIcon,
  BadgeCheckIcon,
  ClockIcon,
  Loader2Icon,
  RefreshCwIcon,
} from 'lucide-react';
import { useState } from 'react';
import { toast } from 'react-toastify';

const TRUST_STYLES: Record<HandleTrust, string> = {
  verified:
    'border-green-600/40 text-green-700 dark:border-green-400/40 dark:text-green-400',
  outdated:
    'border-dashed border-amber-500/70 text-amber-700 dark:text-amber-400',
  changed: 'border-dashed border-destructive/70 text-destructive',
};

function TrustIcon({
  trust,
  className,
}: {
  trust: HandleTrust;
  className?: string;
}) {
  const Icon =
    trust === 'verified'
      ? BadgeCheckIcon
      : trust === 'outdated'
        ? ClockIcon
        : AlertTriangleIcon;

  return <Icon className={cn('h-3.5 w-3.5 shrink-0', className)} aria-hidden />;
}

function trustLabel(trust: HandleTrust) {
  switch (trust) {
    case 'verified':
      return t`Verified`;
    case 'outdated':
      return t`Not verified recently`;
    case 'changed':
      return t`Changed hands`;
  }
}

export interface AddressHandlesProps {
  address: string | null | undefined;
  /** A smaller label, for tables and dense lists. */
  compact?: boolean;
  className?: string;
}

/**
 * Labels an address with the handles Sage already knows resolve to it. The
 * most trustworthy handle is shown, and every handle is listed in a popover
 * with how recently it was verified. Handles which weren't verified recently,
 * or changed hands, are styled differently and never called verified.
 *
 * Nothing is shown for addresses without known handles, and the address
 * itself is always shown by the caller, since a handle never replaces it.
 */
export function AddressHandles({
  address,
  compact = false,
  className,
}: AddressHandlesProps) {
  const handles = useAddressHandles(address);

  if (!address || handles.length === 0) return null;

  const [primary, ...others] = handles;
  const trust = handleTrust(primary);
  const name = primary.handle;
  const label = trustLabel(trust);
  const count = others.length;

  return (
    <Popover>
      <PopoverTrigger asChild>
        <button
          type='button'
          className={cn(
            'inline-flex max-w-full items-center gap-1 rounded-md border bg-background font-medium leading-none hover:bg-accent focus:outline-none focus-visible:ring-2 focus-visible:ring-ring',
            compact
              ? 'max-w-[12rem] px-1 py-0.5 text-xs'
              : 'px-1.5 py-1 text-xs',
            TRUST_STYLES[trust],
            className,
          )}
          title={t`@${name} (${label})`}
          aria-label={
            count > 0
              ? plural(count, {
                  one: `@${name} (${label}) and # more handle. Show all handles for this address`,
                  other: `@${name} (${label}) and # more handles. Show all handles for this address`,
                })
              : t`@${name} (${label}). Show handle details`
          }
          onClick={(event) => event.stopPropagation()}
        >
          <TrustIcon trust={trust} />
          <span className='truncate'>@{name}</span>
          {count > 0 && (
            <span className='shrink-0 text-muted-foreground'>+{count}</span>
          )}
        </button>
      </PopoverTrigger>
      <PopoverContent
        className='w-80 space-y-3 p-3'
        align='start'
        onClick={(event) => event.stopPropagation()}
      >
        <div className='space-y-1'>
          <div className='text-sm font-medium'>
            {handles.length > 1 ? (
              <Trans>Known handles for this address</Trans>
            ) : (
              <Trans>Known handle for this address</Trans>
            )}
          </div>
          <div className='flex items-start gap-1'>
            <span className='font-mono text-xs break-all text-muted-foreground'>
              {address}
            </span>
            <CopyButton
              value={address}
              className='h-4 w-4 shrink-0'
              onCopy={() => toast.success(t`Address copied to clipboard`)}
            />
          </div>
        </div>

        <div className='max-h-72 space-y-2 overflow-y-auto'>
          {handles.map((record) => (
            <HandleDetails key={record.handle} record={record} />
          ))}
        </div>

        <p className='text-xs text-muted-foreground'>
          <Trans>
            Only handles you&apos;ve looked up, or that this wallet owns, are
            shown. Handles can change hands, so check the address before relying
            on one.
          </Trans>
        </p>
      </PopoverContent>
    </Popover>
  );
}

function HandleDetails({ record }: { record: HandleRecord }) {
  const [verifying, setVerifying] = useState(false);
  const trust = handleTrust(record);
  const name = record.handle;
  const verifiedAt = formatTimestamp(record.last_verified_at, 'short', 'short');
  const updatedHeight = record.updated_height.toLocaleString();
  const changedFrom = record.changed_from ?? record.previous_address;

  const verify = () => {
    setVerifying(true);

    commands
      .resolveHandle({ handle: name })
      .then(({ handle }) => {
        if (handle.address === record.address) {
          toast.success(t`@${name} was verified`);
        } else {
          toast.warn(t`@${name} now resolves to a different address`);
        }
      })
      .catch((error: BackendError) => toast.error(handleErrorMessage(error)))
      // Verifying can move the handle to another address, or mark it as no
      // longer verified, so every label, including the names of contacts
      // saved as the handle, is looked up again.
      .finally(() => {
        setVerifying(false);
        refreshAddressLabels();
      });
  };

  return (
    <div className='space-y-1 rounded-md border p-2 text-xs'>
      <div className='flex items-center justify-between gap-2'>
        <div
          className={cn(
            'flex min-w-0 items-center gap-1 font-medium',
            TRUST_STYLES[trust],
          )}
        >
          <TrustIcon trust={trust} />
          <span className='truncate'>@{name}</span>
        </div>
        <span className={cn('shrink-0', TRUST_STYLES[trust])}>
          {trustLabel(trust)}
        </span>
      </div>

      <div className='text-muted-foreground'>
        {trust === 'outdated' ? (
          <Trans>
            Last verified {verifiedAt}. It may resolve somewhere else now.
          </Trans>
        ) : (
          <Trans>Last verified {verifiedAt}</Trans>
        )}
      </div>

      {trust === 'changed' && changedFrom && (
        <div className='text-destructive'>
          <Trans>
            This handle previously resolved to{' '}
            <span className='font-mono break-all'>{changedFrom}</span>, so it
            may belong to someone else now.
          </Trans>
        </div>
      )}

      <div className='text-muted-foreground'>
        <Trans>Last updated at height {updatedHeight}</Trans>
      </div>

      {trust === 'outdated' && (
        <Button
          variant='outline'
          size='sm'
          className='h-7 px-2 text-xs'
          disabled={verifying}
          onClick={verify}
        >
          {verifying ? (
            <Loader2Icon className='mr-1 h-3 w-3 animate-spin' aria-hidden />
          ) : (
            <RefreshCwIcon className='mr-1 h-3 w-3' aria-hidden />
          )}
          <Trans>Verify again</Trans>
        </Button>
      )}
    </div>
  );
}

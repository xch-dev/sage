import { commands, Error as BackendError, HandleRecord } from '@/bindings';
import { useDebounce } from '@/hooks/useDebounce';
import { isHandleInput, isValidHandle } from '@/lib/handles';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import {
  AlertTriangleIcon,
  CheckCircleIcon,
  ClockIcon,
  Loader2Icon,
  XCircleIcon,
} from 'lucide-react';
import { useEffect, useState } from 'react';

type Resolution =
  | { state: 'loading' }
  | { state: 'resolved'; record: HandleRecord }
  | { state: 'failed'; error: BackendError };

function errorMessage(error: BackendError) {
  const reason = error.reason;

  switch (error.kind) {
    case 'not_found':
      return t`This handle isn't registered, or has expired`;
    case 'internal':
      return t`This handle couldn't be verified right now: ${reason}`;
    default:
      return t`This handle couldn't be verified: ${reason}`;
  }
}

/**
 * Shows what a handle recipient resolves to while it's being entered. This is
 * only a preview: the backend resolves and verifies the handle again when the
 * transaction is created, and again right before it's submitted.
 */
export function HandleResolution({ value }: { value: string }) {
  const handle = value.trim();
  const debounced = useDebounce(handle, 400);
  const [resolution, setResolution] = useState<Resolution | null>(null);

  const active = isHandleInput(debounced) && isValidHandle(debounced);

  useEffect(() => {
    if (!active) {
      setResolution(null);
      return;
    }

    let cancelled = false;
    setResolution({ state: 'loading' });

    commands
      .resolveHandle({ handle: debounced })
      .then((response) => {
        if (!cancelled) {
          setResolution({ state: 'resolved', record: response.handle });
        }
      })
      .catch((error: BackendError) => {
        if (!cancelled) setResolution({ state: 'failed', error });
      });

    return () => {
      cancelled = true;
    };
  }, [active, debounced]);

  if (!isHandleInput(handle) || handle !== debounced || !resolution) {
    return null;
  }

  if (resolution.state === 'loading') {
    return (
      <div className='flex items-center gap-1.5 text-xs text-muted-foreground'>
        <Loader2Icon className='h-3.5 w-3.5 animate-spin' aria-hidden='true' />
        <Trans>Resolving {handle}</Trans>
      </div>
    );
  }

  if (resolution.state === 'failed') {
    return (
      <div className='flex items-start gap-1.5 text-xs text-destructive'>
        <XCircleIcon className='h-3.5 w-3.5 mt-0.5 shrink-0' />
        <span className='break-words'>{errorMessage(resolution.error)}</span>
      </div>
    );
  }

  const { record } = resolution;
  const changedFrom = record.changed_from;
  const updatedHeight = record.updated_height.toLocaleString();

  return (
    <div className='space-y-1 text-xs'>
      <div className='flex items-start gap-1.5'>
        {record.cached ? (
          <ClockIcon className='h-3.5 w-3.5 mt-0.5 shrink-0 text-yellow-600' />
        ) : (
          <CheckCircleIcon className='h-3.5 w-3.5 mt-0.5 shrink-0 text-green-600' />
        )}
        <div className='min-w-0'>
          <span className='text-muted-foreground'>
            {record.cached ? (
              <Trans>Resolves to (verified recently, offline)</Trans>
            ) : (
              <Trans>Verified, resolves to</Trans>
            )}
          </span>{' '}
          <span className='font-mono break-all'>{record.address}</span>
          <div className='text-muted-foreground'>
            <Trans>Last updated at height {updatedHeight}</Trans>
          </div>
        </div>
      </div>

      {changedFrom && (
        <div className='flex items-start gap-1.5 text-destructive'>
          <AlertTriangleIcon className='h-3.5 w-3.5 mt-0.5 shrink-0' />
          <span className='break-words'>
            <Trans>
              This handle changed since you last used it. It previously resolved
              to <span className='font-mono break-all'>{changedFrom}</span>.
              You&apos;ll be asked to confirm the new address.
            </Trans>
          </span>
        </div>
      )}
    </div>
  );
}

/** Previews each distinct handle in a list of recipients. */
export function HandleResolutionList({ values }: { values: string[] }) {
  const handles = [...new Set(values.map((value) => value.trim()))].filter(
    isHandleInput,
  );

  if (handles.length === 0) return null;

  return (
    <div className='space-y-2'>
      {handles.map((handle) => (
        <div key={handle}>
          <div className='text-xs font-medium'>{handle}</div>
          <HandleResolution value={handle} />
        </div>
      ))}
    </div>
  );
}

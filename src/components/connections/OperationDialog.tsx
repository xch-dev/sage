import { commands, type OperationReview } from '@/bindings';
import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from '@/components/ui/dialog';
import { Trans } from '@lingui/react/macro';
import { useEffect, useState } from 'react';

export function OperationDialog({
  request,
  unavailable,
  onClose,
  onChanged,
}: {
  request: OperationReview;
  unavailable: boolean;
  onClose: () => void;
  onChanged: () => Promise<void>;
}) {
  const [confirmed, setConfirmed] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [now, setNow] = useState(Date.now());
  useEffect(() => {
    const timer = setInterval(() => setNow(Date.now()), 1000);
    return () => clearInterval(timer);
  }, []);
  const expired = now >= request.expiresAt * 1000;
  const decide = async (approved: boolean) => {
    if (busy || unavailable || expired) return;
    setBusy(true);
    setError(null);
    try {
      await commands.connectionsDecideOperation(request.id, approved);
      onClose();
      await onChanged();
    } catch (err) {
      setError(String(err));
    } finally {
      setBusy(false);
    }
  };
  return (
    <Dialog
      open
      onOpenChange={(open) => {
        if (!open && !busy) onClose();
      }}
    >
      <DialogContent onInteractOutside={(e) => e.preventDefault()}>
        <DialogTitle>
          <Trans>Approve RPC request?</Trans>
        </DialogTitle>
        <DialogDescription>
          <Trans>
            This connection has permission to request this operation. Your
            confirmation authorizes this exact request once. Transactions may
            transfer assets or create binding offers.
          </Trans>
        </DialogDescription>
        <dl className='grid grid-cols-[auto_1fr] gap-x-4 gap-y-2 text-sm'>
          <dt>
            <Trans>Connection</Trans>
          </dt>
          <dd className='break-all font-medium'>{request.name}</dd>
          <dt>
            <Trans>Operation</Trans>
          </dt>
          <dd className='break-all font-mono'>{request.method}</dd>
          <dt>
            <Trans>Network</Trans>
          </dt>
          <dd>{request.network}</dd>
          <dt>
            <Trans>Wallet</Trans>
          </dt>
          <dd>{request.fingerprint ?? <Trans>Global</Trans>}</dd>
          <dt>
            <Trans>Expires</Trans>
          </dt>
          <dd>{new Date(request.expiresAt * 1000).toLocaleTimeString()}</dd>
        </dl>
        <p className='text-sm text-muted-foreground'>
          <Trans>
            Review all parameters, including recipients, fees, and automatic
            submission. Amounts use the RPC’s base units. Only approve requests
            you understand.
          </Trans>
        </p>
        <pre
          className='max-h-72 overflow-auto whitespace-pre-wrap break-all rounded border bg-muted p-3 text-xs'
          aria-label='Request parameters'
        >
          {request.parameters}
        </pre>
        <label className='flex items-start gap-2 text-sm'>
          <input
            type='checkbox'
            checked={confirmed}
            disabled={busy || expired || unavailable}
            onChange={(e) => setConfirmed(e.target.checked)}
          />
          <Trans>I reviewed this request and authorize it once.</Trans>
        </label>
        {(error || unavailable || expired) && (
          <p role='alert' className='text-sm text-destructive'>
            {expired ? (
              <Trans>This request has expired.</Trans>
            ) : (
              error || <Trans>Connection status is unavailable.</Trans>
            )}
          </p>
        )}
        <DialogFooter>
          <Button
            variant='outline'
            disabled={busy || expired || unavailable}
            onClick={() => void decide(false)}
          >
            <Trans>Reject request</Trans>
          </Button>
          <Button
            disabled={busy || !confirmed || expired || unavailable}
            onClick={() => void decide(true)}
          >
            <Trans>Approve once</Trans>
          </Button>
        </DialogFooter>
        <p className='text-xs text-muted-foreground'>
          <Trans>
            Closing defers this request. Find pending requests in Settings →
            Permissions.
          </Trans>
        </p>
      </DialogContent>
    </Dialog>
  );
}

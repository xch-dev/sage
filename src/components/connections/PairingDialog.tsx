import {
  commands,
  type CredentialCapability,
  type PairingInfo,
  type WalletInfo,
} from '@/bindings';
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
import { GrantFields, validGrant } from './GrantFields';

export function PairingDialog({
  request,
  capabilities,
  unavailable,
  onClose,
  onChanged,
}: {
  request: PairingInfo;
  capabilities: CredentialCapability[];
  unavailable: boolean;
  onClose: () => void;
  onChanged: () => Promise<void>;
}) {
  const [grant, setGrant] = useState(request.grant);
  const [wallets, setWallets] = useState<WalletInfo[]>([]);
  const [loaded, setLoaded] = useState(false);
  const [confirmed, setConfirmed] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [now, setNow] = useState(Date.now());
  useEffect(() => {
    let live = true;
    commands
      .getWallets({})
      .then((result) => {
        if (live) {
          setWallets(result.wallets);
          setLoaded(true);
        }
      })
      .catch(() => {
        if (live)
          setError(
            'Wallets could not be loaded. Close and review this request again.',
          );
      });
    const timer = setInterval(() => setNow(Date.now()), 1000);
    return () => {
      live = false;
      clearInterval(timer);
    };
  }, []);
  const expired = now >= request.expiresAt * 1000;
  const decide = async (approve: boolean) => {
    if (busy || expired || unavailable) return;
    setBusy(true);
    setError(null);
    try {
      await commands.connectionsDecidePairing(
        request.id,
        approve ? grant : null,
      );
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
          <Trans>Allow a new connection?</Trans>
        </DialogTitle>
        <DialogDescription>
          <Trans>
            The name below is supplied by the requesting client and is not
            verified. Only approve a request you started.
          </Trans>
        </DialogDescription>
        <div className='rounded-md border bg-muted p-3 text-center'>
          <p className='text-sm'>
            <Trans>Match this code in the requesting client</Trans>
          </p>
          <p className='my-2 font-mono text-2xl tracking-widest'>
            {request.code}
          </p>
          <p className='text-xs'>
            <Trans>Expires</Trans>:{' '}
            {new Date(request.expiresAt * 1000).toLocaleTimeString()}
          </p>
        </div>
        <GrantFields
          grant={grant}
          onChange={setGrant}
          capabilities={capabilities}
          wallets={wallets}
          requested={request.grant}
          disabled={busy || expired || unavailable}
        />
        <label className='flex items-start gap-2 text-sm'>
          <input
            type='checkbox'
            checked={confirmed}
            disabled={busy || expired}
            onChange={(e) => setConfirmed(e.target.checked)}
          />
          <Trans>I started this request and the codes match.</Trans>
        </label>
        {(error || unavailable || expired) && (
          <p role='alert' className='text-sm text-destructive'>
            {expired ? (
              <Trans>This request has expired.</Trans>
            ) : (
              error || (
                <Trans>
                  Connection status is unavailable. Approval is disabled.
                </Trans>
              )
            )}
          </p>
        )}
        <DialogFooter>
          <Button
            variant='outline'
            disabled={busy || expired || unavailable}
            onClick={() => void decide(false)}
          >
            <Trans>Reject</Trans>
          </Button>
          <Button
            disabled={
              busy ||
              !loaded ||
              !confirmed ||
              !validGrant(grant) ||
              expired ||
              unavailable
            }
            onClick={() => void decide(true)}
          >
            <Trans>Approve connection</Trans>
          </Button>
        </DialogFooter>
        <p className='text-xs text-muted-foreground'>
          <Trans>
            Approval lets this client collect its credential once. Closing this
            dialog leaves the request pending; review it again in Settings →
            Permissions.
          </Trans>
        </p>
      </DialogContent>
    </Dialog>
  );
}

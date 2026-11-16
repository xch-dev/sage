import {
  commands,
  type ConnectionGrant,
  type CredentialInfo,
  type IssuedCredential,
  type WalletInfo,
} from '@/bindings';
import { useConnections } from '@/contexts/ConnectionsContext';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import {
  Dialog,
  DialogContent,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from '@/components/ui/dialog';
import { Trans } from '@lingui/react/macro';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import { useEffect, useState } from 'react';
import { GrantFields, validGrant } from './GrantFields';

type Action =
  | { kind: 'create' }
  | { kind: 'rotate' | 'revoke'; credential: CredentialInfo };
export function matchesCredential(credential: CredentialInfo, query: string) {
  return [
    credential.grant.name,
    credential.id,
    ...credential.grant.capabilities,
  ].some((value) => value.toLowerCase().includes(query));
}

export function ConnectionsSettings({
  category,
  query,
  wallets,
}: {
  category: string;
  query: string;
  wallets: WalletInfo[];
}) {
  const connections = useConnections();
  const [action, setAction] = useState<Action | null>(null);
  const [port, setPort] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  if (!connections || category === 'apps') return null;
  const { view, refresh, review } = connections;
  if (!view)
    return (
      <p role='status' className='text-sm'>
        {connections.error || <Trans>Loading connections…</Trans>}
      </p>
    );
  const currentPort = port ?? String(view.port);
  const changeServer = async () => {
    const parsed = Number(currentPort);
    if (
      !view.running &&
      (!Number.isInteger(parsed) || parsed < 1024 || parsed > 65535)
    ) {
      setError('Enter a port between 1024 and 65535.');
      return;
    }
    setBusy(true);
    setError(null);
    try {
      await commands.connectionsSetServer(
        !view.running,
        view.running ? view.port : parsed,
      );
      await refresh();
    } catch (err) {
      setError(String(err));
    } finally {
      setBusy(false);
    }
  };
  const credentials = view.credentials.filter(
    (c) =>
      (category === 'all' ||
        c.grant.kind === (category === 'agents' ? 'agent' : 'service')) &&
      matchesCredential(c, query),
  );
  return (
    <div className='space-y-3'>
      <section className='space-y-3 rounded-lg border p-4'>
        <div className='flex flex-wrap items-center justify-between gap-3'>
          <div>
            <h4 className='font-medium'>
              <Trans>Local service and agent access</Trans>
            </h4>
            <p className='text-sm text-muted-foreground'>
              <Trans>
                Each connection gets its own credential and RPC permissions.
              </Trans>
            </p>
          </div>
          <Button
            size='sm'
            disabled={!!connections.error}
            onClick={() => setAction({ kind: 'create' })}
          >
            <Trans>Create credential</Trans>
          </Button>
        </div>
        <p className='text-sm'>
          {view.running ? (
            <Trans>Listening on this computer</Trans>
          ) : (
            <Trans>Listener stopped</Trans>
          )}{' '}
          · <code>https://127.0.0.1:{view.port}</code>
        </p>
        <p className='text-xs text-muted-foreground'>
          <Trans>
            Enable this listener to use credentials or receive connection
            requests from native clients on this computer. It starts
            automatically with Sage while enabled. Websites cannot use it
            directly.
          </Trans>
        </p>
        <div className='flex items-end gap-3'>
          <label className='block space-y-1 text-sm'>
            <span>
              <Trans>Port</Trans>
            </span>
            <Input
              className='w-28'
              type='number'
              min={1024}
              max={65535}
              value={currentPort}
              disabled={busy || view.running}
              onChange={(e) => setPort(e.target.value)}
            />
          </label>
          <Button
            variant='outline'
            disabled={busy}
            onClick={() => void changeServer()}
          >
            {view.running ? (
              <Trans>Stop listener</Trans>
            ) : (
              <Trans>Enable listener</Trans>
            )}
          </Button>
          {connections.error && (
            <Button variant='outline' onClick={() => void refresh()}>
              <Trans>Refresh</Trans>
            </Button>
          )}
        </div>
        <details className='text-xs text-muted-foreground'>
          <summary className='cursor-pointer'>
            <Trans>Connection instructions</Trans>
          </summary>
          <div className='mt-2 space-y-2'>
            <p>
              <Trans>
                Clients must trust Sage’s public server certificate at this
                path. It is created when you first enable the listener. Share
                the certificate only, never the server key.
              </Trans>
            </p>
            <code className='block break-all'>{view.certificatePath}</code>
            <p>
              <Trans>
                Send the credential in the Authorization: Bearer header. Never
                place it in a URL. Stopping the listener does not revoke
                credentials.
              </Trans>
            </p>
          </div>
        </details>
        {(error || connections.error) && (
          <p role='alert' className='text-sm text-destructive'>
            {error || connections.error}
          </p>
        )}
        <details className='text-xs text-muted-foreground'>
          <summary className='cursor-pointer'>
            <Trans>Owner-only administration</Trans> (
            {view.ownerOnlyMethods.length})
          </summary>
          <p className='my-2'>
            <Trans>
              These commands require owner credentials and cannot be granted to
              services or agents.
            </Trans>
          </p>
          <ul className='grid gap-1 sm:grid-cols-2'>
            {view.ownerOnlyMethods.map((method) => (
              <li key={method}>
                <code>{method}</code>
              </li>
            ))}
          </ul>
        </details>
        {view.operations.map((request) => (
          <div
            key={request.id}
            className='flex items-center justify-between gap-2 rounded border p-3 text-sm'
          >
            <span>
              {request.name} · {request.method}
            </span>
            <Button
              size='sm'
              variant='outline'
              onClick={() => review(request.id)}
            >
              <Trans>Review operation</Trans>
            </Button>
          </div>
        ))}
        {view.pending.map((request) => (
          <div
            key={request.id}
            className='flex items-center justify-between gap-2 rounded border p-3 text-sm'
          >
            <span>
              <Trans>Pending request</Trans>: {request.grant.name} ·{' '}
              <code>{request.code}</code>
            </span>
            <Button
              size='sm'
              variant='outline'
              onClick={() => review(request.id)}
            >
              <Trans>Review request</Trans>
            </Button>
          </div>
        ))}
      </section>
      {credentials.map((credential) => (
        <details key={credential.id} className='rounded-lg border bg-card'>
          <summary className='cursor-pointer p-4'>
            <span className='font-medium'>{credential.grant.name}</span>
            <span className='ml-2 text-sm text-muted-foreground'>
              {credential.grant.kind === 'agent' ? (
                <Trans>Agent</Trans>
              ) : (
                <Trans>External service</Trans>
              )}
            </span>
          </summary>
          <div className='space-y-3 border-t p-4 text-sm'>
            <p>
              {credential.grant.walletScope.kind === 'allWallets' ? (
                <Trans>All wallets, including wallets added later</Trans>
              ) : (
                credential.grant.walletScope.fingerprints
                  .map(
                    (fp) =>
                      `${wallets.find((w) => w.fingerprint === fp)?.name ?? 'Wallet'} (${fp})`,
                  )
                  .join(', ')
              )}
            </p>
            <ul className='space-y-2'>
              {credential.grant.capabilities.map((key) => {
                const cap = view.capabilities.find((c) => c.key === key);
                return (
                  <li key={key}>
                    {cap?.label ?? key}
                    {cap?.approval && (
                      <span className='ml-2 text-xs text-muted-foreground'>
                        <Trans>Confirmation required</Trans>
                      </span>
                    )}
                    <p className='text-xs text-muted-foreground'>
                      {cap?.description}
                    </p>
                  </li>
                );
              })}
            </ul>
            <p className='text-xs text-muted-foreground'>
              <Trans>Created</Trans>:{' '}
              {new Date(credential.createdAt * 1000).toLocaleString()}
              <br />
              <Trans>Identity</Trans>:{' '}
              <code className='break-all'>{credential.id}</code>
            </p>
            <div className='flex gap-2'>
              <Button
                size='sm'
                variant='outline'
                disabled={!!connections.error}
                onClick={() => setAction({ kind: 'rotate', credential })}
              >
                <Trans>Replace credential</Trans>
              </Button>
              <Button
                size='sm'
                variant='destructive'
                disabled={!!connections.error}
                onClick={() => setAction({ kind: 'revoke', credential })}
              >
                <Trans>Revoke access</Trans>
              </Button>
            </div>
          </div>
        </details>
      ))}
      {action && (
        <CredentialDialog
          connections={connections}
          action={action}
          wallets={wallets}
          onClose={() => setAction(null)}
        />
      )}
    </div>
  );
}

function CredentialDialog({
  connections,
  action,
  wallets,
  onClose,
}: {
  connections: NonNullable<ReturnType<typeof useConnections>>;
  action: Action;
  wallets: WalletInfo[];
  onClose: () => void;
}) {
  const setManaging = connections.setManaging;
  useEffect(() => {
    setManaging(true);
    return () => setManaging(false);
  }, [setManaging]);
  const [grant, setGrant] = useState<ConnectionGrant>({
    name: '',
    kind: 'service',
    capabilities: [],
    walletScope: { kind: 'selectedWallets', fingerprints: [] },
  });
  const [issued, setIssued] = useState<IssuedCredential | null>(null);
  const [busy, setBusy] = useState(false);
  const [copied, setCopied] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const submit = async () => {
    if (busy || connections.error) return;
    setBusy(true);
    setError(null);
    try {
      if (action.kind === 'revoke') {
        await commands.connectionsRevoke(action.credential.id);
        onClose();
      } else
        setIssued(
          action.kind === 'create'
            ? await commands.connectionsCreate({
                ...grant,
                name: grant.name.trim(),
              })
            : await commands.connectionsRotate(action.credential.id),
        );
      await connections.refresh();
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
          {issued ? (
            <Trans>Save your credential</Trans>
          ) : action.kind === 'create' ? (
            <Trans>Create credential</Trans>
          ) : action.kind === 'rotate' ? (
            <Trans>Replace credential?</Trans>
          ) : (
            <Trans>Revoke access?</Trans>
          )}
        </DialogTitle>
        <DialogDescription>
          {issued ? (
            <Trans>
              This secret is shown only once. Store it securely before closing;
              Sage stores only its hash. Anyone with it can use the granted
              permissions.
            </Trans>
          ) : action.kind === 'create' ? (
            <Trans>
              Choose the client’s identity, wallets, and RPC permissions. No
              access is selected by default.
            </Trans>
          ) : action.kind === 'rotate' ? (
            <Trans>
              The current credential will stop working immediately. Update the
              client with the new secret; its permissions stay the same.
            </Trans>
          ) : (
            <Trans>
              This client’s credential will stop authorizing new requests
              immediately. A request already in progress may finish.
            </Trans>
          )}
        </DialogDescription>
        {issued ? (
          <>
            <p className='font-medium'>{issued.credential.grant.name}</p>
            <textarea
              aria-label='New credential'
              readOnly
              rows={3}
              className='w-full break-all rounded-md border bg-muted p-3 font-mono text-xs'
              value={issued.token}
              spellCheck={false}
            />
            <Button
              variant='outline'
              onClick={() => {
                void writeText(issued.token)
                  .then(() => setCopied(true))
                  .catch(() =>
                    setError(
                      'Could not copy. Select and copy the credential manually.',
                    ),
                  );
              }}
            >
              {copied ? <Trans>Copied</Trans> : <Trans>Copy credential</Trans>}
            </Button>
            <p className='text-xs text-muted-foreground'>
              <Trans>
                Use Authorization: Bearer followed by this secret. Clear it from
                your clipboard after saving it.
              </Trans>
            </p>
            {!connections.view?.running && (
              <p className='text-sm'>
                <Trans>
                  Enable the local listener in Permissions before connecting.
                </Trans>
              </p>
            )}
          </>
        ) : action.kind === 'create' ? (
          <GrantFields
            grant={grant}
            onChange={setGrant}
            capabilities={connections.view?.capabilities ?? []}
            wallets={wallets}
            disabled={busy}
          />
        ) : (
          <p className='font-medium'>{action.credential.grant.name}</p>
        )}
        {error && (
          <p role='alert' className='text-sm text-destructive'>
            {error}
          </p>
        )}
        <DialogFooter>
          <Button variant='outline' disabled={busy} onClick={onClose}>
            {issued ? <Trans>Done</Trans> : <Trans>Cancel</Trans>}
          </Button>
          {!issued && (
            <Button
              variant={action.kind === 'revoke' ? 'destructive' : 'default'}
              disabled={
                busy ||
                !!connections.error ||
                (action.kind === 'create' && !validGrant(grant))
              }
              onClick={() => void submit()}
            >
              {action.kind === 'create' ? (
                <Trans>Create</Trans>
              ) : action.kind === 'rotate' ? (
                <Trans>Replace</Trans>
              ) : (
                <Trans>Revoke</Trans>
              )}
            </Button>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

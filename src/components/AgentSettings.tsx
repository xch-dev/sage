import {
  AgentServerStatus,
  commands,
  SageAgent,
  SageAppCapabilityDefinitionView,
  SageAppWalletScope,
  UserBridgeCapability,
} from '@/bindings';
import { AgentPermissionsPanel } from '@/components/AgentPermissionsPanel';
import { CopyBox } from '@/components/CopyBox';
import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Switch } from '@/components/ui/switch';
import { CustomError } from '@/contexts/ErrorContext';
import { useWallet } from '@/contexts/WalletContext';
import { useBiometric } from '@/hooks/useBiometric';
import { useErrors } from '@/hooks/useErrors';
import {
  defaultAgentWalletScope,
  hasSelectableWallet,
  hasUnsavedAgentChanges,
  initialAgentCapabilities,
} from '@/lib/agentPermissions';
import { formatTimestamp } from '@/lib/utils';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { useCallback, useEffect, useRef, useState } from 'react';
import type { WalletScopeEditorWallet } from 'sage-app-ui';

const DEFAULT_PORT = 9258;
const MIN_PORT = 1024;
const MAX_PORT = 65535;

function isValidPort(value: string) {
  if (!/^\d+$/.test(value)) return false;
  const port = Number(value);
  return port >= MIN_PORT && port <= MAX_PORT;
}

function capabilitySummary(count: number) {
  return count === 1 ? t`1 capability` : t`${count} capabilities`;
}

function lastUsedSummary(lastUsedAtMs: number | null) {
  if (!lastUsedAtMs) return t`Never used`;
  const when = formatTimestamp(
    Math.floor(lastUsedAtMs / 1000),
    'short',
    'short',
  );
  return t`Last used ${when}`;
}

function scopeSummary(scope: SageAppWalletScope, currentFingerprint?: number) {
  if (scope.kind === 'allWallets') return t`All wallets`;
  if (
    scope.fingerprints.length === 1 &&
    scope.fingerprints[0] === currentFingerprint
  ) {
    return t`This wallet`;
  }
  const count = scope.fingerprints.length;
  return count === 1 ? t`1 wallet` : t`${count} wallets`;
}

export function AgentSettings() {
  const { addError } = useErrors();
  const { promptIfEnabled } = useBiometric();
  const { wallet } = useWallet();

  const [status, setStatus] = useState<AgentServerStatus | null>(null);
  const [portInput, setPortInput] = useState(String(DEFAULT_PORT));
  const [rpcRunning, setRpcRunning] = useState(false);
  const [agents, setAgents] = useState<SageAgent[]>([]);
  const [definitions, setDefinitions] = useState<
    SageAppCapabilityDefinitionView[]
  >([]);

  const [dialogOpen, setDialogOpen] = useState(false);
  const [editing, setEditing] = useState<SageAgent | null>(null);
  const [revoking, setRevoking] = useState<SageAgent | null>(null);

  // One slot per poll, so one poll succeeding doesn't re-arm the other's error.
  const lastPollErrors = useRef({ status: '', rpc: '' });

  const refreshAgents = useCallback(() => {
    commands.appsListAgents().then(setAgents).catch(addError);
  }, [addError]);

  useEffect(() => {
    // Polling repeats every second, so only report an error when it changes.
    const reportPollError = (poll: 'status' | 'rpc') => (error: unknown) => {
      const customError = error as CustomError;
      if (lastPollErrors.current[poll] === customError.reason) return;
      lastPollErrors.current[poll] = customError.reason;
      addError(customError);
    };
    const pollRpcRunning = () =>
      commands
        .isRpcRunning()
        .then((running) => {
          lastPollErrors.current.rpc = '';
          setRpcRunning(running);
        })
        .catch(reportPollError('rpc'));

    commands
      .appsGetAgentServerStatus()
      .then((s) => {
        setStatus(s);
        setPortInput(String(s.port));
      })
      .catch(addError);
    commands.appsListAgentCapabilities().then(setDefinitions).catch(addError);
    refreshAgents();
    pollRpcRunning();

    const interval = setInterval(() => {
      commands
        .appsGetAgentServerStatus()
        .then((s) => {
          lastPollErrors.current.status = '';
          setStatus((prev) =>
            prev && prev.running === s.running && prev.enabled === s.enabled
              ? prev
              : s,
          );
        })
        .catch(reportPollError('status'));
      pollRpcRunning();
    }, 1000);

    return () => clearInterval(interval);
  }, [addError, refreshAgents]);

  const portValid = isValidPort(portInput);

  const applySettings = async (enabled: boolean, port: number) => {
    if (enabled && !(await promptIfEnabled())) return;

    try {
      setStatus(await commands.appsSetAgentServerSettings(enabled, port));
    } catch (error) {
      addError(error as CustomError);
      commands.appsGetAgentServerStatus().then(setStatus).catch(addError);
    }
  };

  const confirmRevoke = async () => {
    if (!revoking) return;
    const id = revoking.id;
    setRevoking(null);
    try {
      await commands.appsRevokeAgent(id);
      refreshAgents();
    } catch (error) {
      addError(error as CustomError);
    }
  };

  const port = status?.port ?? DEFAULT_PORT;
  const revokingName = revoking?.name ?? '';
  const portChanged =
    status !== null && portValid && Number(portInput) !== port;

  return (
    <div className='divide-y rounded-md border bg-card text-card-foreground overflow-hidden'>
      <div className='p-3'>
        <h3 className='text-sm font-medium'>
          <Trans>Agents</Trans>
        </h3>
      </div>
      <div className='divide-y'>
        <div className='p-3 space-y-3'>
          <div className='flex items-center justify-between gap-4'>
            <div className='flex items-center gap-3'>
              <div
                className={`h-3 w-3 rounded-full ${
                  status?.running ? 'bg-green-500' : 'bg-red-500'
                }`}
              />
              <span className='font-medium'>
                {status?.running ? (
                  <Trans>Running</Trans>
                ) : (
                  <Trans>Stopped</Trans>
                )}
              </span>
            </div>
            <Switch
              checked={status?.enabled ?? false}
              disabled={status === null || (!status.enabled && !portValid)}
              onCheckedChange={(checked) =>
                applySettings(checked, portValid ? Number(portInput) : port)
              }
              aria-label={t`Enable agent server`}
            />
          </div>
          <div className='text-sm text-muted-foreground'>
            <Trans>
              Lets AI agents connect to this wallet over MCP on 127.0.0.1.
            </Trans>
          </div>
          <div className='flex items-end gap-2'>
            <div className='space-y-1'>
              <Label htmlFor='agent-port'>
                <Trans>Port</Trans>
              </Label>
              <Input
                id='agent-port'
                className='w-32'
                inputMode='numeric'
                value={portInput}
                onChange={(e) => setPortInput(e.target.value)}
                aria-invalid={!portValid}
              />
            </div>
            {portChanged && (
              <Button
                size='sm'
                onClick={() =>
                  applySettings(status?.enabled ?? false, Number(portInput))
                }
              >
                <Trans>Apply</Trans>
              </Button>
            )}
          </div>
          {!portValid && (
            <div className='text-sm text-destructive'>
              <Trans>
                Port must be between {MIN_PORT} and {MAX_PORT}.
              </Trans>
            </div>
          )}
          {rpcRunning && (
            <div className='rounded-md border border-amber-500/50 bg-amber-500/10 p-2 text-sm text-amber-600 dark:text-amber-400'>
              <Trans>
                The RPC server is also running. Its certificate on disk gives
                full wallet access and bypasses agent approvals.
              </Trans>
            </div>
          )}
        </div>

        <div className='p-3 space-y-3'>
          <div className='flex items-center justify-end'>
            <Button
              size='sm'
              onClick={() => {
                setEditing(null);
                setDialogOpen(true);
              }}
            >
              <Trans>Add agent</Trans>
            </Button>
          </div>
          {agents.length === 0 ? (
            <div className='text-sm text-muted-foreground'>
              <Trans>No agents yet.</Trans>
            </div>
          ) : (
            <div className='space-y-2'>
              {agents.map((agent) => (
                <div
                  key={agent.id}
                  className='flex flex-col sm:flex-row sm:items-center justify-between gap-2 rounded-md border p-2'
                >
                  <div className='min-w-0'>
                    <div className='font-medium truncate'>{agent.name}</div>
                    <div className='text-sm text-muted-foreground'>
                      {scopeSummary(agent.walletScope, wallet?.fingerprint)}
                      {' · '}
                      {capabilitySummary(agent.capabilities.length)}
                      {' · '}
                      {lastUsedSummary(agent.lastUsedAtMs)}
                    </div>
                  </div>
                  <div className='flex gap-2'>
                    <Button
                      variant='outline'
                      size='sm'
                      onClick={() => {
                        setEditing(agent);
                        setDialogOpen(true);
                      }}
                    >
                      <Trans>Edit</Trans>
                    </Button>
                    <Button
                      variant='destructive'
                      size='sm'
                      onClick={() => setRevoking(agent)}
                    >
                      <Trans>Revoke</Trans>
                    </Button>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>

      <AgentDialog
        open={dialogOpen}
        setOpen={setDialogOpen}
        agent={editing}
        definitions={definitions}
        port={port}
        currentFingerprint={wallet?.fingerprint}
        onChanged={refreshAgents}
      />

      <Dialog
        open={revoking !== null}
        onOpenChange={(open) => !open && setRevoking(null)}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>
              <Trans>Revoke agent</Trans>
            </DialogTitle>
            <DialogDescription>
              <Trans>
                Revoke {revokingName}? Its token will stop working immediately.
              </Trans>
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant='outline' onClick={() => setRevoking(null)}>
              <Trans>Cancel</Trans>
            </Button>
            <Button variant='destructive' onClick={confirmRevoke}>
              <Trans>Revoke</Trans>
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}

interface AgentDialogProps {
  open: boolean;
  setOpen: (open: boolean) => void;
  agent: SageAgent | null;
  definitions: SageAppCapabilityDefinitionView[];
  port: number;
  currentFingerprint?: number;
  onChanged: () => void;
}

function AgentDialog({
  open,
  setOpen,
  agent,
  definitions,
  port,
  currentFingerprint,
  onChanged,
}: AgentDialogProps) {
  const { addError } = useErrors();
  const { promptIfEnabled } = useBiometric();

  const [name, setName] = useState('');
  // Null until edited, so a new agent follows the definitions even if they load late.
  const [editedCapabilities, setEditedCapabilities] = useState<
    UserBridgeCapability[] | null
  >(null);
  const [walletScope, setWalletScope] = useState<SageAppWalletScope>(
    defaultAgentWalletScope(),
  );
  const [wallets, setWallets] = useState<WalletScopeEditorWallet[]>([]);
  const [walletsLoaded, setWalletsLoaded] = useState(false);
  const [pending, setPending] = useState(false);
  const [token, setToken] = useState<string | null>(null);

  // Reset the form whenever the dialog is opened.
  useEffect(() => {
    if (!open) return;
    setToken(null);
    setPending(false);
    setName(agent?.name ?? '');
    setEditedCapabilities(null);
    setWalletScope(
      agent ? agent.walletScope : defaultAgentWalletScope(currentFingerprint),
    );
    setWalletsLoaded(false);
    commands
      .getWallets({})
      .then(({ wallets }) => {
        setWallets(
          wallets.map(({ fingerprint, name, emoji }) => ({
            fingerprint,
            name,
            emoji,
          })),
        );
        setWalletsLoaded(true);
      })
      .catch(addError);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open, agent]);

  const capabilities =
    editedCapabilities ?? initialAgentCapabilities(agent, definitions);
  const walletSelected = hasSelectableWallet(walletScope, wallets);

  const submit = async () => {
    if (!(await promptIfEnabled())) return;

    try {
      setPending(true);
      if (agent) {
        await commands.appsUpdateAgent(
          agent.id,
          name.trim(),
          capabilities,
          walletScope,
        );
        onChanged();
        setOpen(false);
      } else {
        const created = await commands.appsCreateAgent(
          name.trim(),
          capabilities,
          walletScope,
        );
        setToken(created.token);
        onChanged();
      }
    } catch (error) {
      addError(error as CustomError);
    } finally {
      setPending(false);
    }
  };

  const rotate = async () => {
    if (!agent) return;
    if (!(await promptIfEnabled())) return;

    try {
      setPending(true);
      setToken(await commands.appsRotateAgentToken(agent.id));
      onChanged();
    } catch (error) {
      addError(error as CustomError);
    } finally {
      setPending(false);
    }
  };

  const valid = name.trim().length > 0 && walletSelected;

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogContent
        className='max-h-[90vh] overflow-y-auto'
        {...(token ? { 'aria-describedby': undefined } : {})}
        // The token is shown once, so a stray Escape or outside click must not dismiss it.
        onEscapeKeyDown={(e) => token && e.preventDefault()}
        onInteractOutside={(e) => token && e.preventDefault()}
      >
        <DialogHeader>
          <DialogTitle>
            {agent ? <Trans>Edit agent</Trans> : <Trans>Add agent</Trans>}
          </DialogTitle>
          {!token && (
            <DialogDescription>
              <Trans>
                Choose which wallets and capabilities this agent can use.
              </Trans>
            </DialogDescription>
          )}
        </DialogHeader>

        {token ? (
          <TokenDisplay token={token} port={port} />
        ) : (
          <div className='space-y-4'>
            <div className='space-y-1'>
              <Label htmlFor='agent-name'>
                <Trans>Name</Trans>
              </Label>
              <Input
                id='agent-name'
                value={name}
                onChange={(e) => setName(e.target.value)}
              />
            </div>

            <AgentPermissionsPanel
              definitions={definitions}
              capabilities={capabilities}
              onCapabilitiesChange={setEditedCapabilities}
              walletScope={walletScope}
              onWalletScopeChange={setWalletScope}
              wallets={wallets}
              disabled={pending}
            />

            {walletsLoaded && !walletSelected && (
              <div className='text-sm text-destructive'>
                <Trans>
                  Select at least one wallet, or choose All wallets.
                </Trans>
              </div>
            )}
          </div>
        )}

        <DialogFooter className='gap-2'>
          {token ? (
            <Button onClick={() => setOpen(false)}>
              <Trans>Done</Trans>
            </Button>
          ) : (
            <>
              {agent && (
                <Button
                  variant='outline'
                  disabled={
                    pending ||
                    hasUnsavedAgentChanges(
                      agent,
                      name,
                      editedCapabilities,
                      walletScope,
                    )
                  }
                  title={t`Save or cancel your changes before rotating the token.`}
                  onClick={rotate}
                >
                  <Trans>Rotate token</Trans>
                </Button>
              )}
              <Button variant='outline' onClick={() => setOpen(false)}>
                <Trans>Cancel</Trans>
              </Button>
              <Button disabled={!valid || pending} onClick={submit}>
                {agent ? <Trans>Save</Trans> : <Trans>Create</Trans>}
              </Button>
            </>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

function TokenDisplay({ token, port }: { token: string; port: number }) {
  const url = `http://127.0.0.1:${port}/mcp`;
  const cliSnippet = `claude mcp add --transport http sage ${url} --header "Authorization: Bearer ${token}"`;
  const jsonSnippet = JSON.stringify({
    mcpServers: {
      sage: {
        type: 'http',
        url,
        headers: { Authorization: `Bearer ${token}` },
      },
    },
  });

  return (
    <div className='space-y-4'>
      <div className='text-sm font-medium text-amber-600 dark:text-amber-400'>
        <Trans>This token is shown only once.</Trans>
      </div>
      <div className='space-y-1'>
        <Label>
          <Trans>Token</Trans>
        </Label>
        <CopyBox title={t`Copy token`} value={token} />
      </div>
      <div className='space-y-1'>
        <Label>
          <Trans>Claude Code</Trans>
        </Label>
        <CopyBox title={t`Copy command`} value={cliSnippet} />
      </div>
      <div className='space-y-1'>
        <Label>
          <Trans>MCP client config (JSON)</Trans>
        </Label>
        <CopyBox title={t`Copy config`} value={jsonSnippet} />
      </div>
    </div>
  );
}

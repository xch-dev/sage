import {
  AgentServerStatus,
  commands,
  SageAgent,
  SageAppCapabilityDefinitionView,
  SageAppWalletScope,
  UserBridgeCapability,
  WalletInfo,
} from '@/bindings';
import { AgentPermissionsPanel } from '@/components/AgentPermissionsPanel';
import { CopyBox } from '@/components/CopyBox';
import {
  ServerStatusIndicator,
  SettingsSection,
} from '@/components/SettingsSection';
import { Alert, AlertDescription } from '@/components/ui/alert';
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
  DEFAULT_AGENT_PORT,
  defaultAgentWalletScope,
  hasSelectableWallet,
  hasUnsavedAgentChanges,
  initialAgentCapabilities,
  isValidAgentPort,
  MAX_AGENT_PORT,
  MIN_AGENT_PORT,
  mcpClientSnippets,
} from '@/lib/agentPermissions';
import { formatTimestamp } from '@/lib/utils';
import { plural, t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { useCallback, useEffect, useState } from 'react';

const REFRESH_INTERVAL_MS = 5000;

function capabilitySummary(count: number) {
  return plural(count, { one: '# capability', other: '# capabilities' });
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
  return plural(scope.fingerprints.length, {
    one: '# wallet',
    other: '# wallets',
  });
}

export function AgentSettings() {
  const { addError } = useErrors();
  const { promptIfEnabled } = useBiometric();
  const { wallet } = useWallet();

  const [status, setStatus] = useState<AgentServerStatus | null>(null);
  const [portInput, setPortInput] = useState(String(DEFAULT_AGENT_PORT));
  const [rpcRunning, setRpcRunning] = useState(false);
  const [agents, setAgents] = useState<SageAgent[]>([]);
  const [definitions, setDefinitions] = useState<
    SageAppCapabilityDefinitionView[]
  >([]);

  const [dialogOpen, setDialogOpen] = useState(false);
  const [editing, setEditing] = useState<SageAgent | null>(null);
  const [revoking, setRevoking] = useState<SageAgent | null>(null);

  const refreshAgents = useCallback(() => {
    commands.appsListAgents().then(setAgents).catch(addError);
  }, [addError]);

  useEffect(() => {
    commands
      .appsGetAgentServerStatus()
      .then((s) => {
        setStatus(s);
        setPortInput(String(s.port));
      })
      .catch(addError);
    commands.appsListAgentCapabilities().then(setDefinitions).catch(addError);
    refreshAgents();
    commands.isRpcRunning().then(setRpcRunning).catch(addError);

    // Agents are used, and the RPC server toggled, outside this component. A failure here was
    // already reported by the first load, so it isn't repeated on every tick.
    const interval = setInterval(() => {
      commands
        .appsListAgents()
        .then(setAgents)
        .catch(() => undefined);
      commands
        .isRpcRunning()
        .then(setRpcRunning)
        .catch(() => undefined);
    }, REFRESH_INTERVAL_MS);

    return () => clearInterval(interval);
  }, [addError, refreshAgents]);

  const portValid = isValidAgentPort(portInput);

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

  const port = status?.port ?? DEFAULT_AGENT_PORT;
  const revokingName = revoking?.name ?? '';
  const startError = status?.startError ?? '';
  const portChanged =
    status !== null && portValid && Number(portInput) !== port;

  return (
    <SettingsSection title={t`Agents`}>
      <div className='p-3 space-y-3'>
        <div className='flex items-center justify-between gap-4'>
          <ServerStatusIndicator running={status?.running ?? false} />
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
              Port must be between {MIN_AGENT_PORT} and {MAX_AGENT_PORT}.
            </Trans>
          </div>
        )}
        {status?.startError && (
          <div className='text-sm text-destructive'>
            <Trans>
              The agent server couldn&apos;t start: {startError}. Turn it off
              and on again to retry.
            </Trans>
          </div>
        )}
        {rpcRunning && (
          <Alert variant='warning'>
            <AlertDescription>
              <Trans>
                The RPC server is also running. Its certificate on disk gives
                full wallet access and bypasses agent approvals.
              </Trans>
            </AlertDescription>
          </Alert>
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
    </SettingsSection>
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
  // Null until the wallets load, so that no wallet is reported missing before then.
  const [wallets, setWallets] = useState<WalletInfo[] | null>(null);
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
    setWallets(null);
    commands
      .getWallets({})
      .then((data) => setWallets(data.wallets))
      .catch(addError);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open, agent]);

  const capabilities =
    editedCapabilities ?? initialAgentCapabilities(agent, definitions);
  const walletSelected = hasSelectableWallet(walletScope, wallets ?? []);

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
      // The save may have failed because a wallet was deleted, which changes the saved scope.
      onChanged();
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
              wallets={wallets ?? []}
              disabled={pending}
            />

            {wallets !== null && !walletSelected && (
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
  const snippets = mcpClientSnippets(token, port);

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
        <CopyBox title={t`Copy command`} value={snippets.claudeCode} />
      </div>
      <div className='space-y-1'>
        <Label>
          <Trans>MCP client config (JSON)</Trans>
        </Label>
        <CopyBox title={t`Copy config`} value={snippets.json} />
      </div>
    </div>
  );
}

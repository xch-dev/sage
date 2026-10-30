import {
  commands,
  events,
  VaultMemberKind,
  VaultRecord,
  VaultSignersRecord,
  VaultState,
} from '@/bindings';
import Container from '@/components/Container';
import { CopyBox } from '@/components/CopyBox';
import Header from '@/components/Header';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { LoadingButton } from '@/components/ui/loading-button';
import { Textarea } from '@/components/ui/textarea';
import { VaultRecoveryFields } from '@/components/VaultRecoveryFields';
import { CustomError } from '@/contexts/ErrorContext';
import { useWallet } from '@/contexts/WalletContext';
import { useErrors } from '@/hooks/useErrors';
import {
  EMPTY_TIMELOCK,
  TimelockInput,
  vaultRecoveryKey,
} from '@/lib/vaultRecovery';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { KeyRound, RotateCcw } from 'lucide-react';
import { useCallback, useEffect, useState } from 'react';
import { toast } from 'react-toastify';

function stateLabel(state: VaultState): string {
  switch (state) {
    case 'unsynced':
      return t`Syncing`;
    case 'active':
      return t`Active`;
    case 'recovery_pending':
      return t`Recovery pending`;
    case 'melted':
      return t`Melted`;
  }
}

function memberKindLabel(kind: VaultMemberKind): string {
  switch (kind) {
    case 'bls':
      return t`BLS key`;
    case 'secp256r1':
      return t`Secure element key`;
    case 'secp256k1':
      return t`secp256k1 key`;
    case 'passkey':
      return t`Passkey`;
    case 'vault':
      return t`Vault`;
    case 'custom':
      return t`Custom`;
  }
}

function formatDuration(seconds: number): string {
  if (seconds % 86_400 === 0) {
    const days = seconds / 86_400;
    return days === 1 ? t`1 day` : t`${days} days`;
  }
  if (seconds % 3600 === 0) {
    const hours = seconds / 3600;
    return hours === 1 ? t`1 hour` : t`${hours} hours`;
  }
  return t`${seconds} seconds`;
}

function SignersList({ signers }: { signers: VaultSignersRecord }) {
  const threshold = signers.threshold;
  const total = signers.members.length;

  return (
    <div className='space-y-2'>
      <p className='text-sm text-muted-foreground'>
        <Trans>
          {threshold} of {total} must sign
        </Trans>
      </p>
      <ul className='space-y-2'>
        {signers.members.map((member) => (
          <li
            key={`${member.kind}-${member.key}`}
            className='flex flex-wrap items-center gap-2 text-sm'
          >
            <Badge variant='outline'>{memberKindLabel(member.kind)}</Badge>
            <span
              className='font-mono truncate max-w-[16rem]'
              title={member.key}
            >
              {member.key}
            </span>
            {member.signer_fingerprint !== null && (
              <Badge>
                <Trans>Signer attached</Trans>
              </Badge>
            )}
          </li>
        ))}
      </ul>
    </div>
  );
}

/** Shows the open vault wallet's state and config, and manages its signers. */
export default function Vault() {
  const { wallet, setWallet } = useWallet();
  const { addError } = useErrors();

  const [vault, setVault] = useState<VaultRecord | null>(null);
  const [attachOpen, setAttachOpen] = useState(false);
  const [key, setKey] = useState('');
  const [attaching, setAttaching] = useState(false);
  const [rebuildOpen, setRebuildOpen] = useState(false);
  const [recoveryKey, setRecoveryKey] = useState('');
  const [timelock, setTimelock] = useState<TimelockInput>(EMPTY_TIMELOCK);
  const [rebuilding, setRebuilding] = useState(false);
  const recovery = vaultRecoveryKey(recoveryKey, timelock);

  const refresh = useCallback(() => {
    commands
      .getVault({})
      .then((data) => setVault(data.vault))
      .catch(addError);
  }, [addError]);

  useEffect(() => {
    refresh();

    const unlisten = events.syncEvent.listen((event) => {
      const type = event.payload.type;

      if (type === 'coin_state' || type === 'puzzle_batch_synced') {
        refresh();
      }
    });

    return () => {
      unlisten.then((u) => u());
    };
  }, [refresh]);

  const rebuildConfig = async () => {
    if (!recovery) return;

    setRebuilding(true);

    try {
      await commands.rebuildVaultConfig({ recovery });
      toast.success(t`Vault config rebuilt`);

      setRecoveryKey('');
      setTimelock(EMPTY_TIMELOCK);
      setRebuildOpen(false);
      refresh();
    } catch (error) {
      addError(error as CustomError);
    } finally {
      setRebuilding(false);
    }
  };

  const attachSigner = async () => {
    if (!wallet) return;

    setAttaching(true);

    try {
      const result = await commands.attachVaultSigner({
        fingerprint: wallet.fingerprint,
        key: key.trim(),
      });

      toast.success(
        result.custody
          ? t`Custody signer attached`
          : t`Recovery signer attached`,
      );

      setKey('');
      setAttachOpen(false);

      const data = await commands.getWallet({});
      setWallet(data.wallet);
      refresh();
    } catch (error) {
      addError(error as CustomError);
    } finally {
      setAttaching(false);
    }
  };

  const config = vault?.config ?? null;
  const recoveryStartedAt = vault?.recovery_started_at
    ? new Date(vault.recovery_started_at * 1000).toLocaleString()
    : null;
  const recoveryDuration = config
    ? formatDuration(config.recovery_timelock)
    : null;

  return (
    <>
      <Header title={t`Vault`} />

      <Container className='flex flex-col gap-4 max-w-screen-lg'>
        <Card>
          <CardHeader>
            <CardTitle className='text-lg font-medium flex items-center gap-2'>
              <Trans>Vault</Trans>
              {vault && (
                <Badge variant='outline'>{stateLabel(vault.state)}</Badge>
              )}
            </CardTitle>
            <CardDescription>
              <Trans>
                This vault is read-only for now. Spending from it will come in a
                future update.
              </Trans>
            </CardDescription>
          </CardHeader>
          <CardContent className='space-y-4'>
            {vault && (
              <CopyBox
                title={t`Launcher ID`}
                value={vault.launcher_id}
                truncateMiddle
              />
            )}
            {vault?.state === 'recovery_pending' && (
              <p className='text-sm'>
                {recoveryStartedAt ? (
                  <Trans>
                    Recovery started on {recoveryStartedAt}. Unless the current
                    custody cancels it, the vault moves to its new custody once
                    the timelock expires.
                  </Trans>
                ) : (
                  <Trans>
                    Recovery has started. Unless the current custody cancels it,
                    the vault moves to its new custody once the timelock
                    expires.
                  </Trans>
                )}
              </p>
            )}
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className='text-lg font-medium'>
              <Trans>Configuration</Trans>
            </CardTitle>
            <CardDescription>
              {config ? (
                config.source === 'reconstructed' ? (
                  <Trans>
                    Rebuilt from the vault&apos;s spends and its recovery key.
                  </Trans>
                ) : (
                  <Trans>Read from the vault&apos;s memos on chain.</Trans>
                )
              ) : (
                <Trans>
                  This vault&apos;s config isn&apos;t known, since it has no
                  memos on chain. Its balances are still shown, but its signers
                  can&apos;t be checked until its config is rebuilt from its
                  spends with its recovery key.
                </Trans>
              )}
            </CardDescription>
          </CardHeader>
          <CardContent className='space-y-4'>
            {config && (
              <>
                <div className='space-y-2'>
                  <h3 className='font-medium'>
                    <Trans>Custody</Trans>
                  </h3>
                  <SignersList signers={config.custody} />
                </div>
                {config.recovery && (
                  <div className='space-y-2'>
                    <h3 className='font-medium'>
                      <Trans>Recovery</Trans>
                    </h3>
                    <SignersList signers={config.recovery} />
                  </div>
                )}
                <p className='text-sm text-muted-foreground'>
                  <Trans>Recovery takes {recoveryDuration}.</Trans>
                </p>
              </>
            )}
            {!config && (
              <Button variant='outline' onClick={() => setRebuildOpen(true)}>
                <RotateCcw className='h-4 w-4 mr-2' aria-hidden='true' />
                <Trans>Rebuild Config</Trans>
              </Button>
            )}
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className='text-lg font-medium'>
              <Trans>Signers</Trans>
            </CardTitle>
            <CardDescription>
              <Trans>
                Keys in your keychain that sign for this vault. A custody signer
                will be able to spend it, and a recovery signer to start
                recovery if the custody keys are lost.
              </Trans>
            </CardDescription>
          </CardHeader>
          <CardContent className='space-y-4'>
            {vault && vault.signers.length > 0 ? (
              <ul className='space-y-2'>
                {vault.signers.map((signer) => (
                  <li
                    key={signer.fingerprint}
                    className='flex flex-wrap items-center gap-2 text-sm'
                  >
                    <KeyRound className='h-4 w-4' aria-hidden='true' />
                    <span className='font-mono'>{signer.fingerprint}</span>
                    {signer.custody && (
                      <Badge variant='outline'>
                        <Trans>Custody</Trans>
                      </Badge>
                    )}
                    {signer.recovery && (
                      <Badge variant='outline'>
                        <Trans>Recovery</Trans>
                      </Badge>
                    )}
                  </li>
                ))}
              </ul>
            ) : (
              <p className='text-sm text-muted-foreground'>
                <Trans>No signers are attached yet.</Trans>
              </p>
            )}
            <Button
              variant='outline'
              onClick={() => setAttachOpen(true)}
              disabled={!config}
            >
              <KeyRound className='h-4 w-4 mr-2' aria-hidden='true' />
              <Trans>Attach Signer</Trans>
            </Button>
          </CardContent>
        </Card>
      </Container>

      <Dialog open={rebuildOpen} onOpenChange={setRebuildOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>
              <Trans>Rebuild Config</Trans>
            </DialogTitle>
            <DialogDescription>
              <Trans>
                A spend through the vault&apos;s custody reveals its custody
                members, and its recovery key and timelock rebuild the rest. An
                M of N custody is only known once each of its members has signed
                a spend.
              </Trans>
            </DialogDescription>
          </DialogHeader>
          <VaultRecoveryFields
            recoveryKey={recoveryKey}
            onRecoveryKeyChange={setRecoveryKey}
            timelock={timelock}
            onTimelockChange={setTimelock}
          />
          <DialogFooter>
            <Button variant='outline' onClick={() => setRebuildOpen(false)}>
              <Trans>Cancel</Trans>
            </Button>
            <LoadingButton
              loading={rebuilding}
              loadingText={t`Rebuilding`}
              disabled={!recovery}
              onClick={rebuildConfig}
            >
              <Trans>Rebuild</Trans>
            </LoadingButton>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog open={attachOpen} onOpenChange={setAttachOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>
              <Trans>Attach Signer</Trans>
            </DialogTitle>
            <DialogDescription>
              <Trans>
                Enter the mnemonic or secret key of a BLS key that&apos;s a
                custody or recovery member of this vault. It&apos;s saved in
                your keychain.
              </Trans>
            </DialogDescription>
          </DialogHeader>
          <Textarea
            className='resize-none h-24'
            value={key}
            onChange={(event) => setKey(event.target.value)}
            aria-label={t`Signer key`}
          />
          <DialogFooter>
            <Button variant='outline' onClick={() => setAttachOpen(false)}>
              <Trans>Cancel</Trans>
            </Button>
            <LoadingButton
              loading={attaching}
              loadingText={t`Attaching`}
              disabled={key.trim().length === 0}
              onClick={attachSigner}
            >
              <Trans>Attach</Trans>
            </LoadingButton>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  );
}

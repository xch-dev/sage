import {
  commands,
  events,
  VaultKeySource,
  VaultRecord,
  VaultRecoveryFinishBlocker,
  WalletInfo,
} from '@/bindings';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Button } from '@/components/ui/button';
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card';
import { Checkbox } from '@/components/ui/checkbox';
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
import { LoadingButton } from '@/components/ui/loading-button';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { Textarea } from '@/components/ui/textarea';
import { CustomError } from '@/contexts/ErrorContext';
import { useWallet } from '@/contexts/WalletContext';
import { useErrors } from '@/hooks/useErrors';
import { timelockParts } from '@/lib/vaultRecovery';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { ShieldAlert } from 'lucide-react';
import { useCallback, useEffect, useState } from 'react';
import { toast } from 'react-toastify';

/** Where a new vault key comes from, as chosen in the form. */
type KeyChoice = 'keep' | 'mnemonic' | 'derived';

interface KeyInput {
  choice: KeyChoice;
  mnemonic: string;
  backedUp: boolean;
  fingerprint: number | null;
}

const EMPTY_KEY: KeyInput = {
  choice: 'mnemonic',
  mnemonic: '',
  backedUp: false,
  fingerprint: null,
};

function keySource(input: KeyInput): VaultKeySource | null | undefined {
  switch (input.choice) {
    case 'keep':
      return null;
    case 'mnemonic':
      return input.mnemonic && input.backedUp
        ? { type: 'mnemonic', mnemonic: input.mnemonic }
        : undefined;
    case 'derived':
      return input.fingerprint !== null
        ? { type: 'derived', fingerprint: input.fingerprint }
        : undefined;
  }
}

/** Seconds left until a time, measured against the latest block timestamp. */
export function recoverySecondsLeft(vault: VaultRecord): number | null {
  if (vault.recovery_finishes_at == null || vault.chain_timestamp == null) {
    return null;
  }

  return Math.max(0, vault.recovery_finishes_at - vault.chain_timestamp);
}

function blockerLabel(blocker: VaultRecoveryFinishBlocker): string {
  switch (blocker) {
    case 'unknown_custody':
      return t`This recovery didn't reveal the custody it moves the vault to, so it can't be finished here. It can only be cancelled.`;
    case 'not_ours':
      return t`This recovery moves the vault to a key that isn't in this wallet, so Sage won't finish it.`;
    case 'timelock':
      return t`The recovery can be finished once its timelock expires.`;
  }
}

function useBlsWallets(): WalletInfo[] {
  const [wallets, setWallets] = useState<WalletInfo[]>([]);

  useEffect(() => {
    commands
      .getWallets({})
      .then((data) =>
        setWallets(data.wallets.filter((wallet) => wallet.kind === 'bls')),
      )
      .catch(() => setWallets([]));
  }, []);

  return wallets;
}

function KeyFields({
  value,
  onChange,
  allowKeep,
  wallets,
}: {
  value: KeyInput;
  onChange: (value: KeyInput) => void;
  allowKeep: boolean;
  wallets: WalletInfo[];
}) {
  const generate = useCallback(() => {
    commands
      .generateMnemonic({ use_24_words: true })
      .then((data) =>
        onChange({ ...value, mnemonic: data.mnemonic, backedUp: false }),
      )
      .catch(() => undefined);
  }, [onChange, value]);

  useEffect(() => {
    if (value.choice === 'mnemonic' && !value.mnemonic) generate();
  }, [value.choice, value.mnemonic, generate]);

  return (
    <div className='space-y-2'>
      <Select
        value={value.choice}
        onValueChange={(choice) =>
          onChange({ ...value, choice: choice as KeyChoice })
        }
      >
        <SelectTrigger>
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          {allowKeep && (
            <SelectItem value='keep'>
              <Trans>Keep the current key</Trans>
            </SelectItem>
          )}
          <SelectItem value='mnemonic'>
            <Trans>A new mnemonic</Trans>
          </SelectItem>
          {wallets.length > 0 && (
            <SelectItem value='derived'>
              <Trans>A key derived from a wallet</Trans>
            </SelectItem>
          )}
        </SelectContent>
      </Select>

      {value.choice === 'mnemonic' && value.mnemonic && (
        <>
          <Textarea
            readOnly
            value={value.mnemonic}
            className='font-mono text-sm'
          />
          <div className='flex items-center gap-2'>
            <Checkbox
              id='vault-key-backed-up'
              checked={value.backedUp}
              onCheckedChange={(checked) =>
                onChange({ ...value, backedUp: checked === true })
              }
            />
            <Label htmlFor='vault-key-backed-up' className='text-sm'>
              <Trans>I&apos;ve written this mnemonic down somewhere safe</Trans>
            </Label>
          </div>
        </>
      )}

      {value.choice === 'derived' && (
        <Select
          value={value.fingerprint?.toString() ?? ''}
          onValueChange={(fingerprint) =>
            onChange({ ...value, fingerprint: Number(fingerprint) })
          }
        >
          <SelectTrigger>
            <SelectValue placeholder={t`Choose a wallet`} />
          </SelectTrigger>
          <SelectContent>
            {wallets.map((wallet) => (
              <SelectItem
                key={wallet.fingerprint}
                value={wallet.fingerprint.toString()}
              >
                {wallet.name}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      )}
    </div>
  );
}

function FeeFields({
  fee,
  onFee,
  feeWallet,
  onFeeWallet,
  wallets,
}: {
  fee: string;
  onFee: (fee: string) => void;
  feeWallet?: number | null;
  onFeeWallet?: (fingerprint: number) => void;
  wallets: WalletInfo[];
}) {
  return (
    <div className='space-y-2'>
      <Label>
        <Trans>Fee in mojos</Trans>
      </Label>
      <Input
        inputMode='numeric'
        value={fee}
        onChange={(event) => onFee(event.target.value.replace(/\D/g, ''))}
      />
      {onFeeWallet && fee !== '' && fee !== '0' && (
        <>
          <p className='text-sm text-muted-foreground'>
            <Trans>
              A recovery can&apos;t pay its own fee, so it&apos;s paid by one of
              your wallets.
            </Trans>
          </p>
          <Select
            value={feeWallet?.toString() ?? ''}
            onValueChange={(fingerprint) => onFeeWallet(Number(fingerprint))}
          >
            <SelectTrigger>
              <SelectValue placeholder={t`Choose a wallet`} />
            </SelectTrigger>
            <SelectContent>
              {wallets.map((wallet) => (
                <SelectItem
                  key={wallet.fingerprint}
                  value={wallet.fingerprint.toString()}
                >
                  {wallet.name}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </>
      )}
    </div>
  );
}

/**
 * A dialog that previews a vault transaction, then submits it once confirmed.
 * Recovery and rekey transactions are signed by Sage when they're submitted,
 * since they change the vault's custody on purpose.
 */
function ActionDialog({
  open,
  onOpenChange,
  title,
  description,
  children,
  submitLabel,
  disabled,
  destructive,
  run,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  title: string;
  description: string;
  children?: React.ReactNode;
  submitLabel: string;
  disabled?: boolean;
  destructive?: boolean;
  run: () => Promise<void>;
}) {
  const { addError } = useErrors();
  const [pending, setPending] = useState(false);

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className='max-h-[90vh] overflow-y-auto'>
        <DialogHeader>
          <DialogTitle>{title}</DialogTitle>
          <DialogDescription>{description}</DialogDescription>
        </DialogHeader>
        <div className='space-y-4'>{children}</div>
        <DialogFooter>
          <Button variant='outline' onClick={() => onOpenChange(false)}>
            <Trans>Cancel</Trans>
          </Button>
          <LoadingButton
            loading={pending}
            disabled={disabled}
            variant={destructive ? 'destructive' : 'default'}
            onClick={() => {
              setPending(true);
              run()
                .then(() => onOpenChange(false))
                .catch((error) => addError(error as CustomError))
                .finally(() => setPending(false));
            }}
          >
            {submitLabel}
          </LoadingButton>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

/** The recovery and rekey actions of the vault page. */
export function VaultRecoveryCard({
  vault,
  onChanged,
}: {
  vault: VaultRecord;
  onChanged: () => void;
}) {
  const wallets = useBlsWallets();
  const [dialog, setDialog] = useState<
    'start' | 'cancel' | 'finish' | 'rekey' | null
  >(null);

  const [custody, setCustody] = useState<KeyInput>(EMPTY_KEY);
  const [recoveryKey, setRecoveryKey] = useState<KeyInput>({
    ...EMPTY_KEY,
    choice: 'keep',
  });
  const [recoveryMnemonic, setRecoveryMnemonic] = useState('');
  const [timelockHours, setTimelockHours] = useState('');
  const [fee, setFee] = useState('0');
  const [feeWallet, setFeeWallet] = useState<number | null>(null);

  const close = (open: boolean) => {
    if (open) return;
    setDialog(null);
    setCustody(EMPTY_KEY);
    setRecoveryKey({ ...EMPTY_KEY, choice: 'keep' });
    setRecoveryMnemonic('');
    setTimelockHours('');
    setFee('0');
    setFeeWallet(null);
  };

  const recovery = vault.recovery;
  const currentTimelock =
    vault.config?.recovery_timelock != null
      ? formatDuration(vault.config.recovery_timelock)
      : null;
  const hasRecovery = vault.config?.recovery != null;
  const holdsRecoveryKey = vault.signers.some((signer) => signer.recovery);
  const secondsLeft = recoverySecondsLeft(vault);
  const timeLeft = secondsLeft === null ? null : formatDuration(secondsLeft);
  const needsFeeWallet = fee !== '' && fee !== '0' && feeWallet === null;
  const custodySource = keySource(custody);
  const recoverySource = keySource(recoveryKey);
  const timelock =
    timelockHours === '' ? null : Math.round(Number(timelockHours) * 3600);

  if (vault.state !== 'active' && vault.state !== 'recovery_pending') {
    return null;
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle className='text-lg font-medium'>
          <Trans>Recovery</Trans>
        </CardTitle>
        <CardDescription>
          {recovery ? (
            <Trans>
              A recovery is pending. Unless the current custody cancels it, it
              moves the vault to a new custody once its timelock expires.
            </Trans>
          ) : (
            <Trans>
              If the custody key is lost, the recovery key can start a recovery
              to a new key, which the custody can cancel until its timelock
              expires. The custody can also rekey the vault at any time.
            </Trans>
          )}
        </CardDescription>
      </CardHeader>
      <CardContent className='space-y-4'>
        {recovery && (
          <>
            <p className='text-sm'>
              {secondsLeft === null ? (
                <Trans>
                  The time left is shown once the block that started it has
                  synced.
                </Trans>
              ) : secondsLeft > 0 ? (
                <Trans>
                  About {timeLeft} left, measured by block timestamps.
                </Trans>
              ) : (
                <Trans>The timelock has expired.</Trans>
              )}
            </p>
            {recovery.finish_blocker && (
              <p className='text-sm text-muted-foreground'>
                {blockerLabel(recovery.finish_blocker)}
              </p>
            )}
          </>
        )}

        <div className='flex flex-wrap gap-2'>
          {recovery ? (
            <>
              <Button
                variant='destructive'
                disabled={!recovery.can_cancel}
                onClick={() => setDialog('cancel')}
              >
                <Trans>Cancel recovery</Trans>
              </Button>
              <Button
                variant='outline'
                disabled={recovery.finish_blocker !== null}
                onClick={() => setDialog('finish')}
              >
                <Trans>Finish recovery</Trans>
              </Button>
            </>
          ) : (
            <>
              {hasRecovery && (
                <Button variant='outline' onClick={() => setDialog('start')}>
                  <Trans>Start recovery</Trans>
                </Button>
              )}
              <Button variant='outline' onClick={() => setDialog('rekey')}>
                <Trans>Rekey</Trans>
              </Button>
            </>
          )}
        </div>
      </CardContent>

      <ActionDialog
        open={dialog === 'start'}
        onOpenChange={close}
        title={t`Start Recovery`}
        description={t`The recovery key moves the vault to a new custody key once the recovery timelock expires. Until then, the vault can't be spent, and the current custody can cancel the recovery.`}
        submitLabel={t`Start recovery`}
        destructive
        disabled={
          custodySource == null ||
          needsFeeWallet ||
          (!holdsRecoveryKey && !recoveryMnemonic.trim())
        }
        run={async () => {
          if (custodySource == null) return;
          await commands.startVaultRecovery({
            recovery_mnemonic: holdsRecoveryKey
              ? null
              : recoveryMnemonic.trim(),
            custody: custodySource,
            fee: fee || '0',
            fee_fingerprint: feeWallet,
            auto_submit: true,
          });
          toast.success(t`Recovery started`);
          onChanged();
        }}
      >
        {!holdsRecoveryKey && (
          <div className='space-y-2'>
            <Label>
              <Trans>Recovery key mnemonic</Trans>
            </Label>
            <Textarea
              value={recoveryMnemonic}
              onChange={(event) => setRecoveryMnemonic(event.target.value)}
              className='font-mono text-sm'
            />
          </div>
        )}
        <div className='space-y-2'>
          <Label>
            <Trans>New custody key</Trans>
          </Label>
          <KeyFields
            value={custody}
            onChange={setCustody}
            allowKeep={false}
            wallets={wallets}
          />
        </div>
        <FeeFields
          fee={fee}
          onFee={setFee}
          feeWallet={feeWallet}
          onFeeWallet={setFeeWallet}
          wallets={wallets}
        />
      </ActionDialog>

      <ActionDialog
        open={dialog === 'cancel'}
        onOpenChange={close}
        title={t`Cancel Recovery`}
        description={t`The custody key recreates the vault as it was before the recovery started. The fee is paid from the vault.`}
        submitLabel={t`Cancel recovery`}
        destructive
        run={async () => {
          await commands.cancelVaultRecovery({
            fee: fee || '0',
            auto_submit: true,
          });
          toast.success(t`Recovery cancelled`);
          onChanged();
        }}
      >
        <FeeFields fee={fee} onFee={setFee} wallets={wallets} />
      </ActionDialog>

      <ActionDialog
        open={dialog === 'finish'}
        onOpenChange={close}
        title={t`Finish Recovery`}
        description={t`The vault moves to its new custody key, which is already in this wallet.`}
        submitLabel={t`Finish recovery`}
        disabled={needsFeeWallet}
        run={async () => {
          await commands.finishVaultRecovery({
            fee: fee || '0',
            fee_fingerprint: feeWallet,
            auto_submit: true,
          });
          toast.success(t`Recovery finished`);
          onChanged();
        }}
      >
        <FeeFields
          fee={fee}
          onFee={setFee}
          feeWallet={feeWallet}
          onFeeWallet={setFeeWallet}
          wallets={wallets}
        />
      </ActionDialog>

      <ActionDialog
        open={dialog === 'rekey'}
        onOpenChange={close}
        title={t`Rekey Vault`}
        description={t`The custody key recreates the vault with a new config. Anything you don't change is kept, and the vault is upgraded to the latest puzzle version.`}
        submitLabel={t`Rekey`}
        disabled={
          custodySource === undefined ||
          recoverySource === undefined ||
          (timelock !== null && (!(timelock > 0) || timelock < 3600))
        }
        run={async () => {
          if (custodySource === undefined || recoverySource === undefined) {
            return;
          }
          await commands.rekeyVault({
            custody: custodySource,
            recovery: recoverySource,
            recovery_timelock: timelock,
            fee: fee || '0',
            auto_submit: true,
          });
          toast.success(t`Vault rekeyed`);
          // The new config is in the rekey's memo, but a backup restores it if
          // the memo can't be read.
          toast.info(
            t`Once the rekey confirms, export the vault config again to back up the new keys`,
          );
          onChanged();
        }}
      >
        <div className='space-y-2'>
          <Label>
            <Trans>Custody key</Trans>
          </Label>
          <KeyFields
            value={{ ...custody, choice: custody.choice }}
            onChange={setCustody}
            allowKeep
            wallets={wallets}
          />
        </div>
        <div className='space-y-2'>
          <Label>
            <Trans>Recovery key</Trans>
          </Label>
          <KeyFields
            value={recoveryKey}
            onChange={setRecoveryKey}
            allowKeep={hasRecovery}
            wallets={wallets}
          />
        </div>
        <div className='space-y-2'>
          <Label>
            <Trans>Recovery timelock in hours (at least 1)</Trans>
          </Label>
          <Input
            inputMode='decimal'
            placeholder={
              currentTimelock !== null ? t`Keep ${currentTimelock}` : undefined
            }
            value={timelockHours}
            onChange={(event) => setTimelockHours(event.target.value)}
          />
        </div>
        <FeeFields fee={fee} onFee={setFee} wallets={wallets} />
      </ActionDialog>
    </Card>
  );
}

/** Which recoveries the user has been notified about, by the vault's coin. */
const NOTIFIED_KEY = 'sage-vault-recovery-notified';

function notifyOnce(vault: VaultRecord) {
  const key = vault.coin_id;
  if (!key) return;

  try {
    const notified: string[] = JSON.parse(
      localStorage.getItem(NOTIFIED_KEY) ?? '[]',
    );
    if (notified.includes(key)) return;
    localStorage.setItem(
      NOTIFIED_KEY,
      JSON.stringify([...notified.slice(-20), key]),
    );
  } catch {
    // Without storage, the notification may be shown again.
  }

  if (typeof Notification === 'undefined') return;

  const show = () =>
    new Notification(t`Vault recovery started`, {
      body: t`A recovery of your vault was started that this wallet didn't start. Cancel it if it wasn't you.`,
    });

  if (Notification.permission === 'granted') {
    show();
  } else if (Notification.permission !== 'denied') {
    Notification.requestPermission()
      .then((permission) => {
        if (permission === 'granted') show();
      })
      .catch(() => undefined);
  }
}

/**
 * A persistent alert, shown on every page of a vault wallet, for a recovery
 * that this wallet didn't start. The timelock exists so that it can be
 * cancelled in time, so it stays until the recovery is cancelled or finished.
 */
export function VaultRecoveryAlert() {
  const { isVault, wallet } = useWallet();
  const { addError } = useErrors();
  const [vault, setVault] = useState<VaultRecord | null>(null);
  const [cancelling, setCancelling] = useState(false);

  const refresh = useCallback(() => {
    if (!isVault) return setVault(null);
    commands
      .getVault({})
      .then((data) => setVault(data.vault))
      .catch(() => setVault(null));
  }, [isVault]);

  useEffect(() => {
    refresh();
    const unlisten = events.syncEvent.listen((event) => {
      const type = event.payload.type;
      if (type === 'coin_state' || type === 'puzzle_batch_synced') refresh();
    });
    const interval = setInterval(refresh, 60_000);
    return () => {
      clearInterval(interval);
      unlisten.then((stop) => stop());
    };
  }, [refresh, wallet?.fingerprint]);

  const recovery = vault?.recovery;
  const unexpected = vault !== null && recovery && !recovery.started_locally;

  useEffect(() => {
    if (unexpected && vault) notifyOnce(vault);
  }, [unexpected, vault]);

  if (!unexpected || !vault || !recovery) return null;

  const secondsLeft = recoverySecondsLeft(vault);
  const timeLeft = secondsLeft === null ? null : formatDuration(secondsLeft);

  return (
    <Alert variant='destructive' className='mb-4'>
      <ShieldAlert className='h-4 w-4' aria-hidden='true' />
      <AlertTitle>
        <Trans>A recovery of this vault was started</Trans>
      </AlertTitle>
      <AlertDescription className='space-y-2'>
        <p>
          {secondsLeft === null ? (
            <Trans>
              This wallet didn&apos;t start it. If you didn&apos;t start it on
              another device, cancel it before its timelock expires, or the
              vault will move to someone else&apos;s key.
            </Trans>
          ) : (
            <Trans>
              This wallet didn&apos;t start it. If you didn&apos;t start it on
              another device, cancel it within about {timeLeft}, or the vault
              will move to someone else&apos;s key.
            </Trans>
          )}
        </p>
        {recovery.can_cancel ? (
          <LoadingButton
            size='sm'
            variant='destructive'
            loading={cancelling}
            onClick={() => {
              setCancelling(true);
              commands
                .cancelVaultRecovery({ fee: '0', auto_submit: true })
                .then(() => {
                  toast.success(t`Recovery cancelled`);
                  refresh();
                })
                .catch((error) => addError(error as CustomError))
                .finally(() => setCancelling(false));
            }}
          >
            <Trans>Cancel recovery</Trans>
          </LoadingButton>
        ) : (
          <p>
            <Trans>
              Attach the vault&apos;s custody key on the vault page to cancel
              it.
            </Trans>
          </p>
        )}
      </AlertDescription>
    </Alert>
  );
}

/** A duration to the minute, such as "11 hours 59 minutes". */
function formatDuration(totalSeconds: number): string {
  const { days, hours, minutes } = timelockParts(
    Math.max(60, Math.round(totalSeconds / 60) * 60),
  );

  return [
    days === 1 ? t`1 day` : days > 0 ? t`${days} days` : null,
    hours === 1 ? t`1 hour` : hours > 0 ? t`${hours} hours` : null,
    minutes === 1 ? t`1 minute` : minutes > 0 ? t`${minutes} minutes` : null,
  ]
    .filter((part) => part !== null)
    .join(' ');
}

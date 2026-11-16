import { commands, KeyInfo, MintVaultResponse, WalletInfo } from '@/bindings';
import Container from '@/components/Container';
import { EmojiPicker } from '@/components/EmojiPicker';
import Header from '@/components/Header';
import MnemonicDisplay from '@/components/MnemonicDisplay';
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
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { LoadingButton } from '@/components/ui/loading-button';
import { FeeAmountInput } from '@/components/ui/masked-input';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { CustomError } from '@/contexts/ErrorContext';
import { useWallet } from '@/contexts/WalletContext';
import { useErrors } from '@/hooks/useErrors';
import { fromMojos, toMojos } from '@/lib/utils';
import {
  DEFAULT_MINT_TIMELOCK,
  MIN_MINT_TIMELOCK,
  formatDuration,
  timelockParts,
  timelockSeconds,
  TimelockInput,
} from '@/lib/vaultRecovery';
import { fetchState, useWalletState } from '@/state';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import { CopyIcon, RefreshCwIcon, ShieldAlertIcon } from 'lucide-react';
import { useCallback, useEffect, useId, useRef, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { toast } from 'react-toastify';

type Step = 'custody' | 'recovery' | 'timelock' | 'review';

const STEPS: Step[] = ['custody', 'recovery', 'timelock', 'review'];

/** Where a key for the vault comes from: derived from the open wallet's key,
 *  generated, or an existing key in the keychain. */
type KeySource = 'derived' | 'generated' | number;

function initialTimelock(): TimelockInput {
  const parts = timelockParts(DEFAULT_MINT_TIMELOCK);

  return {
    days: parts.days ? String(parts.days) : '',
    hours: parts.hours ? String(parts.hours) : '',
    minutes: parts.minutes ? String(parts.minutes) : '',
  };
}

export default function MintVault() {
  const navigate = useNavigate();
  const { addError } = useErrors();
  const { wallet, setWallet } = useWallet();
  const walletState = useWalletState();
  const precision = walletState.sync.unit.precision;
  const ticker = walletState.sync.unit.ticker;
  const id = useId();

  const [step, setStep] = useState<Step>('custody');
  const [name, setName] = useState('');
  const [emoji, setEmoji] = useState<string | null>(null);
  const [custody, setCustody] = useState<KeySource>('derived');
  const [recovery, setRecovery] = useState<KeySource>('generated');
  const [mnemonic, setMnemonic] = useState('');
  const [backedUp, setBackedUp] = useState(false);
  const [timelock, setTimelock] = useState<TimelockInput>(initialTimelock);
  const [fee, setFee] = useState('');
  const [keys, setKeys] = useState<KeyInfo[]>([]);
  const [wallets, setWallets] = useState<WalletInfo[]>([]);
  const [preview, setPreview] = useState<MintVaultResponse | null>(null);
  const [minting, setMinting] = useState(false);
  const mnemonicRequestId = useRef(0);

  useEffect(() => {
    commands
      .getKeys({})
      .then((data) => setKeys(data.keys.filter((key) => key.has_secrets)))
      .catch(addError);
    commands
      .getWallets({})
      .then((data) => setWallets(data.wallets))
      .catch(addError);
  }, [addError]);

  const loadMnemonic = useCallback(() => {
    const requestId = ++mnemonicRequestId.current;
    setBackedUp(false);

    commands
      .generateMnemonic({ use_24_words: true })
      .then((data) => {
        if (requestId === mnemonicRequestId.current) setMnemonic(data.mnemonic);
      })
      .catch((error) => {
        if (requestId === mnemonicRequestId.current) addError(error);
      });
  }, [addError]);

  useEffect(() => {
    loadMnemonic();
  }, [loadMnemonic]);

  const seconds = timelockSeconds(timelock);
  const timelockValid =
    typeof seconds === 'number' && seconds >= MIN_MINT_TIMELOCK;
  const feeMojos = fee ? toMojos(fee, precision) : '0';

  const keyName = (fingerprint: number) =>
    wallets.find((wallet) => wallet.fingerprint === fingerprint)?.name ??
    t`Key ${fingerprint}`;

  const request = (autoSubmit: boolean) => ({
    name: name.trim() || t`Vault`,
    emoji,
    custody_fingerprint: typeof custody === 'number' ? custody : null,
    recovery_mnemonic: recovery === 'generated' ? mnemonic : null,
    recovery_fingerprint: typeof recovery === 'number' ? recovery : null,
    recovery_timelock: typeof seconds === 'number' ? seconds : 0,
    fee: feeMojos,
    auto_submit: autoSubmit,
  });

  const canContinue = (() => {
    switch (step) {
      case 'custody':
        return true;
      case 'recovery':
        return recovery === 'generated'
          ? mnemonic !== '' && backedUp
          : recovery !== custody;
      case 'timelock':
        return timelockValid;
      case 'review':
        return preview !== null;
    }
  })();

  const next = async () => {
    const index = STEPS.indexOf(step);
    const nextStep = STEPS[index + 1];

    if (nextStep === 'review') {
      setPreview(null);

      try {
        setPreview(await commands.mintVault(request(false)));
      } catch (error) {
        addError(error as CustomError);
        return;
      }
    }

    if (nextStep) setStep(nextStep);
  };

  const back = () => {
    const index = STEPS.indexOf(step);
    if (index === 0) navigate('/wallet');
    else setStep(STEPS[index - 1]);
  };

  const mint = async () => {
    setMinting(true);

    try {
      const response = await commands.mintVault(request(true));

      if (response.fingerprint === null) return;

      toast.success(t`Vault minted. It appears once its transaction confirms.`);

      // The vault wallet is opened right away, and shows that it's minting
      // until the mint confirms.
      await commands.login({ fingerprint: response.fingerprint });
      await fetchState();
      const data = await commands.getWallet({});
      setWallet(data.wallet);
      navigate('/vault?backup=1');
    } catch (error) {
      addError(error as CustomError);
    } finally {
      setMinting(false);
    }
  };

  const keyOptions = (exclude: KeySource) =>
    keys
      .filter((key) => key.fingerprint !== exclude)
      .map((key) => (
        <SelectItem key={key.fingerprint} value={String(key.fingerprint)}>
          {keyName(key.fingerprint)} ({key.fingerprint})
        </SelectItem>
      ));

  const stepNumber = STEPS.indexOf(step) + 1;
  const stepCount = STEPS.length;
  const walletName = wallet?.name ?? t`this wallet`;
  const minimumTimelock = formatDuration(MIN_MINT_TIMELOCK);

  const timelockFields = [
    ['days', t`Days`],
    ['hours', t`Hours`],
    ['minutes', t`Minutes`],
  ] as const;

  return (
    <>
      <Header title={t`Mint Vault`} back={back} />

      <Container className='max-w-xl'>
        <p className='text-sm text-muted-foreground mb-4'>
          <Trans>
            Step {stepNumber} of {stepCount}
          </Trans>
        </p>

        {step === 'custody' && (
          <Card>
            <CardHeader>
              <CardTitle>
                <Trans>Vault and Custody Key</Trans>
              </CardTitle>
              <CardDescription>
                <Trans>
                  A vault is a smart wallet whose coins are spent with its
                  custody key. Its addresses stay the same when its keys change.
                  The vault is paid for by {walletName}.
                </Trans>
              </CardDescription>
            </CardHeader>
            <CardContent className='space-y-4'>
              <div className='space-y-2'>
                <Label htmlFor={`${id}-name`}>
                  <Trans>Vault Name</Trans>
                </Label>
                <Input
                  id={`${id}-name`}
                  value={name}
                  placeholder={t`Vault`}
                  onChange={(event) => setName(event.target.value)}
                />
              </div>

              <div className='space-y-2'>
                <Label>
                  <Trans>Vault Emoji (Optional)</Trans>
                </Label>
                <div>
                  <EmojiPicker
                    value={emoji}
                    onChange={setEmoji}
                    placeholder={t`Choose an emoji`}
                  />
                </div>
              </div>

              <div className='space-y-2'>
                <Label>
                  <Trans>Custody Key</Trans>
                </Label>
                <Select
                  value={String(custody)}
                  onValueChange={(value) =>
                    setCustody(value === 'derived' ? 'derived' : Number(value))
                  }
                >
                  <SelectTrigger aria-label={t`Custody Key`}>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value='derived'>
                      <Trans>A new key derived from this wallet</Trans>
                    </SelectItem>
                    {keyOptions(recovery)}
                  </SelectContent>
                </Select>
                <p className='text-sm text-muted-foreground'>
                  {custody === 'derived' ? (
                    <Trans>
                      The custody key is derived from this wallet&apos;s key, so
                      its mnemonic also backs up the custody key.
                    </Trans>
                  ) : (
                    <Trans>
                      An existing key in the keychain is the custody key.
                    </Trans>
                  )}
                </p>
              </div>
            </CardContent>
          </Card>
        )}

        {step === 'recovery' && (
          <Card>
            <CardHeader>
              <CardTitle>
                <Trans>Recovery Key</Trans>
              </CardTitle>
              <CardDescription>
                <Trans>
                  If the custody key is ever lost, the recovery key can move the
                  vault to a new custody key after the recovery timelock. The
                  custody key can cancel a recovery it didn&apos;t start before
                  then.
                </Trans>
              </CardDescription>
            </CardHeader>
            <CardContent className='space-y-4'>
              <Select
                value={String(recovery)}
                onValueChange={(value) =>
                  setRecovery(
                    value === 'generated' ? 'generated' : Number(value),
                  )
                }
              >
                <SelectTrigger aria-label={t`Recovery Key`}>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value='generated'>
                    <Trans>A new recovery phrase</Trans>
                  </SelectItem>
                  {keyOptions(custody)}
                </SelectContent>
              </Select>

              {recovery === 'generated' ? (
                <>
                  <Alert variant='warning'>
                    <ShieldAlertIcon className='h-4 w-4' />
                    <AlertTitle>
                      <Trans>Write down this recovery phrase</Trans>
                    </AlertTitle>
                    <AlertDescription>
                      <Trans>
                        Sage keeps a copy, so it can start or cancel a recovery,
                        but the recovery key exists for when this device is
                        lost. Keep the phrase offline, somewhere separate from
                        your wallet&apos;s mnemonic.
                      </Trans>
                    </AlertDescription>
                  </Alert>

                  <div>
                    <div className='flex justify-between items-center mb-2'>
                      <Label>
                        <Trans>Recovery Phrase</Trans>
                      </Label>
                      <div>
                        <Button
                          type='button'
                          variant='ghost'
                          size='sm'
                          aria-label={t`Generate another phrase`}
                          onClick={loadMnemonic}
                        >
                          <RefreshCwIcon className='h-4 w-4' />
                        </Button>
                        <Button
                          type='button'
                          variant='ghost'
                          size='sm'
                          aria-label={t`Copy phrase`}
                          disabled={!mnemonic}
                          onClick={() => writeText(mnemonic)}
                        >
                          <CopyIcon className='h-4 w-4' />
                        </Button>
                      </div>
                    </div>
                    <MnemonicDisplay mnemonic={mnemonic} />
                  </div>

                  <div className='flex items-start gap-2'>
                    <Checkbox
                      id={`${id}-backed-up`}
                      checked={backedUp}
                      onCheckedChange={(checked) =>
                        setBackedUp(checked === true)
                      }
                    />
                    <Label
                      htmlFor={`${id}-backed-up`}
                      className='text-sm font-normal leading-snug'
                    >
                      <Trans>
                        I wrote down the recovery phrase and stored it offline.
                      </Trans>
                    </Label>
                  </div>
                </>
              ) : (
                <p className='text-sm text-muted-foreground'>
                  <Trans>
                    An existing key in the keychain is the recovery key. Make
                    sure its mnemonic is backed up offline.
                  </Trans>
                </p>
              )}
            </CardContent>
          </Card>
        )}

        {step === 'timelock' && (
          <Card>
            <CardHeader>
              <CardTitle>
                <Trans>Recovery Timelock and Fee</Trans>
              </CardTitle>
              <CardDescription>
                <Trans>
                  A recovery can only finish once this much time has passed
                  since it started. A longer timelock leaves more time to notice
                  and cancel a recovery you didn&apos;t start.
                </Trans>
              </CardDescription>
            </CardHeader>
            <CardContent className='space-y-4'>
              <div className='grid grid-cols-3 gap-2'>
                {timelockFields.map(([part, label]) => (
                  <div key={part} className='space-y-1'>
                    <Label htmlFor={`${id}-${part}`}>{label}</Label>
                    <Input
                      id={`${id}-${part}`}
                      inputMode='numeric'
                      value={timelock[part]}
                      onChange={(event) =>
                        setTimelock({ ...timelock, [part]: event.target.value })
                      }
                    />
                  </div>
                ))}
              </div>
              {!timelockValid && (
                <p className='text-sm text-destructive'>
                  <Trans>
                    The timelock must be at least {minimumTimelock}.
                  </Trans>
                </p>
              )}

              <div className='space-y-2'>
                <Label htmlFor={`${id}-fee`}>
                  <Trans>Network Fee</Trans>
                </Label>
                <FeeAmountInput
                  id={`${id}-fee`}
                  value={fee}
                  onValueChange={(values) => setFee(values.value)}
                />
              </div>
            </CardContent>
          </Card>
        )}

        {step === 'review' && preview && (
          <Card>
            <CardHeader>
              <CardTitle>
                <Trans>Review</Trans>
              </CardTitle>
              <CardDescription>
                <Trans>
                  Minting the vault costs one mojo, plus the fee. After it
                  confirms, export a backup of the vault&apos;s config, which
                  can import it again even without its memos.
                </Trans>
              </CardDescription>
            </CardHeader>
            <CardContent className='space-y-3 text-sm'>
              <ReviewRow label={t`Name`} value={request(false).name} />
              <ReviewRow
                label={t`Custody Key`}
                value={`${typeof custody === 'number' ? keyName(custody) : t`Derived from this wallet`} (${preview.custody_fingerprint})`}
              />
              <ReviewRow
                label={t`Recovery Key`}
                value={`${typeof recovery === 'number' ? keyName(recovery) : t`New recovery phrase`} (${preview.recovery_fingerprint})`}
              />
              <ReviewRow
                label={t`Recovery Timelock`}
                value={formatDuration(request(false).recovery_timelock)}
              />
              <ReviewRow
                label={t`Fee`}
                value={`${fromMojos(preview.summary.fee, precision)} ${ticker}`}
              />
            </CardContent>
          </Card>
        )}

        <div className='flex justify-between mt-4'>
          <Button variant='outline' onClick={back}>
            <Trans>Back</Trans>
          </Button>
          {step === 'review' ? (
            <LoadingButton
              loading={minting}
              loadingText={t`Minting`}
              disabled={!canContinue}
              onClick={mint}
            >
              <Trans>Mint Vault</Trans>
            </LoadingButton>
          ) : (
            <Button disabled={!canContinue} onClick={next}>
              <Trans>Continue</Trans>
            </Button>
          )}
        </div>
      </Container>
    </>
  );
}

function ReviewRow({ label, value }: { label: string; value: string }) {
  return (
    <div className='flex justify-between gap-4'>
      <span className='text-muted-foreground shrink-0'>{label}</span>
      <span className='font-medium text-right break-all'>{value}</span>
    </div>
  );
}

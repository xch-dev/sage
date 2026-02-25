import { EmojiPicker } from '@/components/EmojiPicker';
import Header from '@/components/Header';
import { MnemonicDisplay } from '@/components/MnemonicDisplay';
import SafeAreaView from '@/components/SafeAreaView';
import { Button } from '@/components/ui/button';
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card';
import {
  Form,
  FormControl,
  FormDescription,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from '@/components/ui/form';
import { DurationInput } from '@/components/DurationInput';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { useErrors } from '@/hooks/useErrors';
import { zodResolver } from '@hookform/resolvers/zod';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { KeyIcon, ShieldIcon } from 'lucide-react';
import { useCallback, useEffect, useState } from 'react';
import { useForm } from 'react-hook-form';
import { useNavigate } from 'react-router-dom';
import { toast } from 'react-toastify';
import * as z from 'zod';
import { commands } from '../bindings';
import Container from '../components/Container';

type KeyType = 'bls' | 'secure-element';

interface WizardState {
  step: number;
  name: string;
  emoji: string | null;
  custodyKeyType: KeyType;
  custodyMnemonic: string;
  recoveryMnemonic: string;
  timelockDays: string;
  timelockHours: string;
  timelockMinutes: string;
}

export default function MintVault() {
  const navigate = useNavigate();

  const [wizard, setWizard] = useState<WizardState>({
    step: 1,
    name: '',
    emoji: null,
    custodyKeyType: 'bls',
    custodyMnemonic: '',
    recoveryMnemonic: '',
    timelockDays: '',
    timelockHours: '12',
    timelockMinutes: '',
  });

  return (
    <SafeAreaView>
      <Header title={t`Mint Vault`} back={() => navigate('/')} />
      <Container>
        <StepIndicator
          current={wizard.step}
          onStepClick={(step) => setWizard((prev) => ({ ...prev, step }))}
        />
        {wizard.step === 1 && (
          <StepWalletInfo wizard={wizard} setWizard={setWizard} />
        )}
        {wizard.step === 2 && (
          <StepCustodyKey wizard={wizard} setWizard={setWizard} />
        )}
        {wizard.step === 3 && (
          <StepRecoveryKey wizard={wizard} setWizard={setWizard} />
        )}
      </Container>
    </SafeAreaView>
  );
}

function StepIndicator({
  current,
  onStepClick,
}: {
  current: number;
  onStepClick: (step: number) => void;
}) {
  const steps = [t`Wallet`, t`Custody`, t`Recovery`];

  return (
    <div className='flex items-center justify-center gap-4 mb-6'>
      {steps.map((label, i) => {
        const step = i + 1;
        const isActive = step === current;
        const isCompleted = step < current;
        const isClickable = step <= current;

        return (
          <div key={label} className='flex items-center gap-2'>
            <button
              type='button'
              className={`flex items-center gap-1.5 ${isClickable ? 'cursor-pointer' : 'cursor-default'}`}
              disabled={!isClickable}
              onClick={() => isClickable && onStepClick(step)}
            >
              <div
                className={`h-7 w-7 rounded-full flex items-center justify-center text-xs font-medium ${
                  isActive
                    ? 'bg-primary text-primary-foreground'
                    : isCompleted
                      ? 'bg-primary/20 text-primary'
                      : 'bg-muted text-muted-foreground'
                }`}
              >
                {step}
              </div>
              <span
                className={`text-sm hidden sm:inline ${isActive ? 'font-medium' : 'text-muted-foreground'}`}
              >
                {label}
              </span>
            </button>
          </div>
        );
      })}
    </div>
  );
}

const step1Schema = z.object({
  name: z.string().min(1),
  emoji: z.string().nullable().optional(),
});

function StepWalletInfo({
  wizard,
  setWizard,
}: {
  wizard: WizardState;
  setWizard: React.Dispatch<React.SetStateAction<WizardState>>;
}) {
  const form = useForm<z.infer<typeof step1Schema>>({
    resolver: zodResolver(step1Schema),
    defaultValues: {
      name: wizard.name,
      emoji: wizard.emoji,
    },
  });

  const onSubmit = (values: z.infer<typeof step1Schema>) => {
    setWizard((prev) => ({
      ...prev,
      step: 2,
      name: values.name,
      emoji: values.emoji ?? null,
    }));
  };

  return (
    <Card className='max-w-xl mx-auto'>
      <CardHeader className='text-center'>
        <CardTitle>
          <Trans>Wallet</Trans>
        </CardTitle>
        <CardDescription>
          <Trans>Choose a name and emoji for your vault wallet.</Trans>
        </CardDescription>
      </CardHeader>
      <CardContent>
        <Form {...form}>
          <form
            onSubmit={form.handleSubmit(onSubmit)}
            className='space-y-4 py-0'
          >
            <FormField
              control={form.control}
              name='name'
              render={({ field }) => (
                <FormItem>
                  <FormLabel>
                    <Trans>Wallet Name</Trans>
                  </FormLabel>
                  <FormControl>
                    <Input placeholder={t`Enter wallet name`} required {...field} />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name='emoji'
              render={({ field }) => (
                <FormItem>
                  <label htmlFor='emoji' className='space-y-0.5'>
                    <FormLabel>
                      <Trans>Wallet Emoji (Optional)</Trans>
                    </FormLabel>
                    <FormControl>
                      <div className='flex items-center gap-2'>
                        <EmojiPicker
                          value={field.value}
                          onChange={field.onChange}
                          placeholder={t`Choose an emoji`}
                        />
                      </div>
                    </FormControl>
                    <FormDescription>
                      <Trans>
                        Choose an emoji to easily identify your wallet
                      </Trans>
                    </FormDescription>
                  </label>
                  <FormMessage />
                </FormItem>
              )}
            />

            <Button type='submit'>
              <Trans>Next</Trans>
            </Button>
          </form>
        </Form>
      </CardContent>
    </Card>
  );
}

function StepCustodyKey({
  wizard,
  setWizard,
}: {
  wizard: WizardState;
  setWizard: React.Dispatch<React.SetStateAction<WizardState>>;
}) {
  const { addError } = useErrors();
  const [mnemonic, setMnemonic] = useState(wizard.custodyMnemonic);

  const loadMnemonic = useCallback(() => {
    commands
      .generateMnemonic({ use_24_words: true })
      .then((data) => setMnemonic(data.mnemonic))
      .catch(addError);
  }, [addError]);

  useEffect(() => {
    if (!mnemonic) {
      loadMnemonic();
    }
  }, [mnemonic, loadMnemonic]);

  const handleNext = () => {
    setWizard((prev) => ({
      ...prev,
      step: 3,
      custodyMnemonic: mnemonic,
    }));
  };

  return (
    <Card className='max-w-xl mx-auto'>
      <CardHeader className='text-center'>
        <CardTitle>
          <Trans>Custody</Trans>
        </CardTitle>
        <CardDescription>
          <Trans>
            Select the type of key used for day-to-day custody of your vault.
          </Trans>
        </CardDescription>
      </CardHeader>
      <CardContent className='space-y-4'>
        <div className='grid grid-cols-2 gap-3'>
          <Button
            type='button'
            variant='outline'
            className={`h-auto py-4 flex flex-col items-center gap-2 ${wizard.custodyKeyType === 'bls' ? 'bg-accent' : ''}`}
            onClick={() =>
              setWizard((prev) => ({ ...prev, custodyKeyType: 'bls' }))
            }
          >
            <KeyIcon className='h-5 w-5' />
            <span>
              <Trans>BLS Key</Trans>
            </span>
          </Button>
          <Button
            type='button'
            variant='outline'
            className='h-auto py-4 flex flex-col items-center gap-2 opacity-50'
            disabled
          >
            <ShieldIcon className='h-5 w-5' />
            <span>
              <Trans>Secure Element</Trans>
            </span>
            <span className='text-xs text-muted-foreground'>
              <Trans>Coming Soon</Trans>
            </span>
          </Button>
        </div>

        {wizard.custodyKeyType === 'bls' && (
          <>
            <div className='mt-3'>
              <MnemonicDisplay
                mnemonic={mnemonic}
                onRegenerate={loadMnemonic}
              />
            </div>

          </>
        )}

        <div className='flex gap-2'>
          <Button
            type='button'
            variant='outline'
            onClick={() => setWizard((prev) => ({ ...prev, step: 1 }))}
          >
            <Trans>Back</Trans>
          </Button>
          <Button type='button' onClick={handleNext} disabled={!mnemonic}>
            <Trans>Next</Trans>
          </Button>
        </div>
      </CardContent>
    </Card>
  );
}

function StepRecoveryKey({
  wizard,
  setWizard,
}: {
  wizard: WizardState;
  setWizard: React.Dispatch<React.SetStateAction<WizardState>>;
}) {
  const { addError } = useErrors();
  const [mnemonic, setMnemonic] = useState(wizard.recoveryMnemonic);

  const loadMnemonic = useCallback(() => {
    commands
      .generateMnemonic({ use_24_words: true })
      .then((data) => setMnemonic(data.mnemonic))
      .catch(addError);
  }, [addError]);

  useEffect(() => {
    if (!mnemonic) {
      loadMnemonic();
    }
  }, [mnemonic, loadMnemonic]);

  const handleCreate = () => {
    toast.success(t`Vault creation is not yet implemented.`);
  };

  return (
    <Card className='max-w-xl mx-auto'>
      <CardHeader className='text-center'>
        <CardTitle>
          <Trans>Recovery</Trans>
        </CardTitle>
        <CardDescription>
          <Trans>
            Set up a recovery key and timelock duration for your vault.
          </Trans>
        </CardDescription>
      </CardHeader>
      <CardContent className='space-y-4'>
        <MnemonicDisplay
          mnemonic={mnemonic}
          label={<Trans>Recovery Mnemonic</Trans>}
          onRegenerate={loadMnemonic}
        />

        <div>
          <Label className='mb-2 block'>
            <Trans>Timelock Duration</Trans>
          </Label>
          <DurationInput
            value={{
              days: wizard.timelockDays,
              hours: wizard.timelockHours,
              minutes: wizard.timelockMinutes,
            }}
            onChange={({ days, hours, minutes }) =>
              setWizard((prev) => ({
                ...prev,
                timelockDays: days,
                timelockHours: hours,
                timelockMinutes: minutes,
              }))
            }
          />
        </div>

        <div className='flex gap-2'>
          <Button
            type='button'
            variant='outline'
            onClick={() =>
              setWizard((prev) => ({
                ...prev,
                step: 2,
                recoveryMnemonic: mnemonic,
              }))
            }
          >
            <Trans>Back</Trans>
          </Button>
          <Button type='button' onClick={handleCreate}>
            <Trans>Create Vault</Trans>
          </Button>
        </div>
      </CardContent>
    </Card>
  );
}

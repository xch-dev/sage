import { EmojiPicker } from '@/components/EmojiPicker';
import Header from '@/components/Header';
import SafeAreaView from '@/components/SafeAreaView';
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
import { Input } from '@/components/ui/input';
import { DurationInput } from '@/components/DurationInput';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';
import { Button } from '@/components/ui/button';
import { zodResolver } from '@hookform/resolvers/zod';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { useForm } from 'react-hook-form';
import { useNavigate } from 'react-router-dom';
import { toast } from 'react-toastify';
import * as z from 'zod';
import Container from '../components/Container';

const formSchema = z.object({
  vaultId: z.string().min(1),
  recoveryKey: z.string().min(1),
  name: z.string().min(1),
  emoji: z.string().nullable().optional(),
  timelockDays: z.string().optional(),
  timelockHours: z.string().optional(),
  timelockMinutes: z.string().optional(),
});

function cleanKey(key: string) {
  return key
    .trim()
    .replace(/[^a-z]/gi, ' ')
    .split(/\s+/)
    .filter((item) => item.length > 0)
    .join(' ')
    .toLowerCase();
}

export default function RecoverVault() {
  const navigate = useNavigate();

  const form = useForm<z.infer<typeof formSchema>>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      timelockDays: '',
      timelockHours: '',
      timelockMinutes: '',
    },
  });

  const onSubmit = (values: z.infer<typeof formSchema>) => {
    // Clean the recovery key before use
    const _cleaned = cleanKey(values.recoveryKey);
    void _cleaned;
    toast.success(t`Vault recovery is not yet implemented.`);
  };

  return (
    <SafeAreaView>
      <Header title={t`Recover Vault`} back={() => navigate('/')} />
      <Container>
        <Card className='max-w-xl mx-auto'>
          <CardHeader className='text-center'>
            <CardTitle>
              <Trans>Recover Vault</Trans>
            </CardTitle>
            <CardDescription>
              <Trans>
                Recover an existing vault using its ID and your recovery key.
              </Trans>
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

                <FormField
                  control={form.control}
                  name='vaultId'
                  render={({ field }) => (
                    <FormItem>
                      <FormLabel>
                        <Trans>Vault ID</Trans>
                      </FormLabel>
                      <FormControl>
                        <Input
                          required
                          placeholder={t`Launcher ID or address`}
                          {...field}
                        />
                      </FormControl>
                      <FormDescription>
                        <Trans>
                          Enter the launcher ID or address of the vault to
                          recover.
                        </Trans>
                      </FormDescription>
                      <FormMessage />
                    </FormItem>
                  )}
                />

                <FormField
                  control={form.control}
                  name='recoveryKey'
                  render={({ field }) => (
                    <FormItem>
                      <FormLabel>
                        <Trans>Recovery Key</Trans>
                      </FormLabel>
                      <FormControl>
                        <Textarea className='resize-none h-20' placeholder={t`Enter recovery seed phrase`} {...field} />
                      </FormControl>
                      <FormDescription>
                        <Trans>
                          Enter your recovery mnemonic seed phrase. Words should
                          be separated by spaces.
                        </Trans>
                      </FormDescription>
                      <FormMessage />
                    </FormItem>
                  )}
                />

                <div>
                  <Label className='mb-2 block'>
                    <Trans>Timelock Duration (Optional)</Trans>
                  </Label>
                  <DurationInput
                    value={{
                      days: form.watch('timelockDays') ?? '',
                      hours: form.watch('timelockHours') ?? '',
                      minutes: form.watch('timelockMinutes') ?? '',
                    }}
                    onChange={({ days, hours, minutes }) => {
                      form.setValue('timelockDays', days);
                      form.setValue('timelockHours', hours);
                      form.setValue('timelockMinutes', minutes);
                    }}
                  />
                </div>

                <Button type='submit'>
                  <Trans>Recover Vault</Trans>
                </Button>
              </form>
            </Form>
          </CardContent>
        </Card>
      </Container>
    </SafeAreaView>
  );
}

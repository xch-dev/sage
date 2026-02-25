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
import { LoadingButton } from '@/components/ui/loading-button';
import { Textarea } from '@/components/ui/textarea';
import { useWallet } from '@/contexts/WalletContext';
import { useErrors } from '@/hooks/useErrors';
import { zodResolver } from '@hookform/resolvers/zod';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { useState } from 'react';
import { useForm } from 'react-hook-form';
import { useNavigate } from 'react-router-dom';
import * as z from 'zod';
import { commands } from '../bindings';
import Container from '../components/Container';
import { fetchState } from '../state';

const formSchema = z.object({
  name: z.string().min(1),
  emoji: z.string().nullable().optional(),
  addresses: z.string().min(1),
});

export default function WatchAddress() {
  const navigate = useNavigate();
  const { addError } = useErrors();
  const { setWallet } = useWallet();
  const [pending, setPending] = useState(false);

  const form = useForm<z.infer<typeof formSchema>>({
    resolver: zodResolver(formSchema),
  });

  const onSubmit = (values: z.infer<typeof formSchema>) => {
    const addresses = values.addresses
      .split('\n')
      .map((a) => a.trim())
      .filter((a) => a.length > 0);

    if (addresses.length === 0) return;

    setPending(true);

    commands
      .importAddresses({
        name: values.name,
        addresses,
        login: true,
        emoji: values.emoji || null,
      })
      .then(async () => {
        await fetchState();
        const data = await commands.getKey({});
        setWallet(data.key);
        navigate('/wallet');
      })
      .catch(addError)
      .finally(() => setPending(false));
  };

  return (
    <SafeAreaView>
      <Header title={t`Watch Address`} back={() => navigate('/')} />
      <Container>
        <Card className='max-w-xl mx-auto'>
          <CardHeader className='text-center'>
            <CardTitle>
              <Trans>Watch Address</Trans>
            </CardTitle>
            <CardDescription>
              <Trans>
                Add addresses to watch in read-only mode. No private keys are
                required.
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
                        <Input
                          placeholder={t`Enter wallet name`}
                          required
                          {...field}
                        />
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
                  name='addresses'
                  render={({ field }) => (
                    <FormItem>
                      <FormLabel>
                        <Trans>Addresses</Trans>
                      </FormLabel>
                      <FormControl>
                        <Textarea
                          className='resize-none h-32'
                          placeholder={t`Enter one address per line`}
                          {...field}
                        />
                      </FormControl>
                      <FormDescription>
                        <Trans>
                          Enter the addresses you want to watch, one per line.
                        </Trans>
                      </FormDescription>
                      <FormMessage />
                    </FormItem>
                  )}
                />

                <LoadingButton
                  type='submit'
                  loading={pending}
                  loadingText={t`Importing`}
                  disabled={!form.formState.isValid}
                >
                  <Trans>Watch</Trans>
                </LoadingButton>
              </form>
            </Form>
          </CardContent>
        </Card>
      </Container>
    </SafeAreaView>
  );
}

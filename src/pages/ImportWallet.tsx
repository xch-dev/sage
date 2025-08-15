import { EmojiPicker } from '@/components/EmojiPicker';
import Header from '@/components/Header';
import SafeAreaView from '@/components/SafeAreaView';
import { Button } from '@/components/ui/button';
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
import { Switch } from '@/components/ui/switch';
import { Textarea } from '@/components/ui/textarea';
import { useWallet } from '@/contexts/WalletContext';
import { useErrors } from '@/hooks/useErrors';
import { zodResolver } from '@hookform/resolvers/zod';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { LoaderCircleIcon } from 'lucide-react';
import { useState } from 'react';
import { useForm } from 'react-hook-form';
import { useNavigate } from 'react-router-dom';
import * as z from 'zod';
import { commands } from '../bindings';
import Container from '../components/Container';
import { fetchState } from '../state';

export default function ImportWallet() {
  const navigate = useNavigate();

  const { addError } = useErrors();
  const { setWallet } = useWallet();

  const [advanced, setAdvanced] = useState(false);
  const [pending, setPending] = useState(false);

  const formSchema = z.object({
    name: z.string(),
    key: z.string(),
    addresses: z.string().refine((value) => {
      const num = parseInt(value);

      return (
        isFinite(num) &&
        Math.floor(num) === num &&
        !isNaN(num) &&
        num >= 0 &&
        num <= 100000
      );
    }),
    emoji: z.string().nullable().optional(),
  });

  const form = useForm<z.infer<typeof formSchema>>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      addresses: '5000',
    },
  });

  const submit = (values: z.infer<typeof formSchema>) => {
    setPending(true);

    commands
      .importKey({
        name: values.name,
        key: values.key,
        derivation_index: parseInt(values.addresses),
        emoji: values.emoji || null,
      })
      .then(fetchState)
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
      <Header title={t`Import Wallet`} back={() => navigate('/')} />
      <Container>
        <Form {...form}>
          <form
            onSubmit={form.handleSubmit(submit)}
            className='space-y-4 max-w-xl mx-auto py-4'
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
                    <Input required {...field} />
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
                  <FormLabel>
                    <Trans>Wallet Emoji (Optional)</Trans>
                  </FormLabel>
                  <FormControl>
                    <EmojiPicker
                      value={field.value}
                      onChange={field.onChange}
                      placeholder='Choose an emoji for your wallet'
                    />
                  </FormControl>
                  <FormDescription>
                    <Trans>
                      Choose an emoji to easily identify your wallet
                    </Trans>
                  </FormDescription>
                  <FormMessage />
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name='key'
              render={({ field }) => (
                <FormItem>
                  <FormLabel>
                    <Trans>Wallet Key</Trans>
                  </FormLabel>
                  <FormControl>
                    <Textarea className='resize-none h-20' {...field} />
                  </FormControl>
                  <FormDescription>
                    <Trans>
                      Enter your mnemonic, private key, or public key above. If
                      it&apos;s a public key, it will be imported as a read-only
                      cold wallet.
                    </Trans>
                  </FormDescription>
                  <FormMessage />
                </FormItem>
              )}
            />

            <div className='flex items-center gap-2 my-4'>
              <label htmlFor='advanced'>
                <Trans>Advanced options</Trans>
              </label>
              <Switch
                id='advanced'
                checked={advanced}
                onCheckedChange={(value) => setAdvanced(value)}
              />
            </div>

            {advanced && (
              <FormField
                control={form.control}
                name='addresses'
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>
                      <Trans>Initial Addresses</Trans>
                    </FormLabel>
                    <FormControl>
                      <Input required {...field} />
                    </FormControl>
                    <FormDescription>
                      <Trans>
                        The initial derivation index to sync to (both hardened
                        and unhardened keys). This is primarily applicable to
                        legacy wallets with either hardened keys or gaps in
                        addresses used.
                      </Trans>
                    </FormDescription>
                    <FormMessage />
                  </FormItem>
                )}
              />
            )}

            <Button type='submit' disabled={pending || !form.formState.isValid}>
              {pending && (
                <LoaderCircleIcon className='mr-2 h-4 w-4 animate-spin' />
              )}
              {pending ? <Trans>Importing</Trans> : <Trans>Import</Trans>}
            </Button>
          </form>
        </Form>
      </Container>
    </SafeAreaView>
  );
}

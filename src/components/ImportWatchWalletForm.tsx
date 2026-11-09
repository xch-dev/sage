import { commands } from '@/bindings';
import { EmojiPicker } from '@/components/EmojiPicker';
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
import { splitAddresses } from '@/lib/walletKind';
import { fetchState } from '@/state';
import { zodResolver } from '@hookform/resolvers/zod';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { useState } from 'react';
import { useForm } from 'react-hook-form';
import { useNavigate } from 'react-router-dom';
import * as z from 'zod';

/** Imports a read-only wallet that watches addresses, without any keys. */
export function ImportWatchWalletForm() {
  const navigate = useNavigate();
  const { addError } = useErrors();
  const { setWallet } = useWallet();
  const [pending, setPending] = useState(false);

  const formSchema = z.object({
    name: z.string().min(1),
    addresses: z.string().refine((value) => splitAddresses(value).length > 0, {
      message: t`Enter at least one address`,
    }),
    emoji: z.string().nullable().optional(),
  });

  const form = useForm<z.infer<typeof formSchema>>({
    resolver: zodResolver(formSchema),
    defaultValues: { name: '', addresses: '' },
    mode: 'onChange',
  });

  const submit = (values: z.infer<typeof formSchema>) => {
    setPending(true);

    commands
      .importWatchWallet({
        name: values.name,
        addresses: splitAddresses(values.addresses),
        emoji: values.emoji || null,
      })
      .then(async () => {
        await fetchState();
        const data = await commands.getWallet({});
        setWallet(data.wallet);
        navigate('/wallet');
      })
      .catch(addError)
      .finally(() => setPending(false));
  };

  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit(submit)} className='space-y-4 py-4'>
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
                <div className='flex items-center gap-2'>
                  <EmojiPicker
                    value={field.value}
                    onChange={field.onChange}
                    placeholder={t`Choose an emoji`}
                  />
                </div>
              </FormControl>
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
                <Textarea className='resize-none h-28' {...field} />
              </FormControl>
              <FormDescription>
                <Trans>
                  Enter one or more addresses, separated by spaces, commas or
                  new lines. The first one is shown as the receive address. You
                  can add or remove addresses later.
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
          <Trans>Import</Trans>
        </LoadingButton>
      </form>
    </Form>
  );
}

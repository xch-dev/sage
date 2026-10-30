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
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { LoadingButton } from '@/components/ui/loading-button';
import { Textarea } from '@/components/ui/textarea';
import { useWallet } from '@/contexts/WalletContext';
import { useErrors } from '@/hooks/useErrors';
import { vaultsEnabled } from '@/lib/features';
import { splitAddresses } from '@/lib/walletKind';
import { fetchState } from '@/state';
import { zodResolver } from '@hookform/resolvers/zod';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { useState } from 'react';
import { useForm } from 'react-hook-form';
import { useNavigate } from 'react-router-dom';
import * as z from 'zod';

interface ImportWatchWalletFormProps {
  /** Called to import the vault that owns a watched address instead. */
  onImportVault?: (launcherId: string) => void;
}

/** Imports a read-only wallet that watches addresses, without any keys. */
export function ImportWatchWalletForm({
  onImportVault,
}: ImportWatchWalletFormProps) {
  const navigate = useNavigate();
  const { addError } = useErrors();
  const { setWallet } = useWallet();
  const [pending, setPending] = useState(false);
  // The vault that owns the address being watched, which can be imported instead.
  const [vault, setVault] = useState<string | null>(null);

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

  const watch = (values: z.infer<typeof formSchema>) => {
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

  const submit = async (values: z.infer<typeof formSchema>) => {
    const addresses = splitAddresses(values.addresses);

    // A single address may belong to a vault, which can be imported with its
    // full history and config instead. If it can't be checked, it's watched.
    if (vaultsEnabled && onImportVault && addresses.length === 1) {
      setPending(true);

      const launcherId = await commands
        .findVault({ address: addresses[0] })
        .then((data) => data.launcher_id)
        .catch(() => null);

      setPending(false);

      if (launcherId) {
        setVault(launcherId);
        return;
      }
    }

    watch(values);
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

        {vault ? (
          <div className='rounded-md border p-4 space-y-3'>
            <p className='text-sm'>
              <Trans>
                This address belongs to a vault. Import the vault to follow its
                config and every address it owns, or watch just this address.
              </Trans>
            </p>
            <div className='flex flex-wrap gap-2'>
              <Button type='button' onClick={() => onImportVault?.(vault)}>
                <Trans>Import Vault</Trans>
              </Button>
              <LoadingButton
                type='button'
                variant='outline'
                loading={pending}
                loadingText={t`Importing`}
                onClick={form.handleSubmit(watch)}
              >
                <Trans>Watch Address</Trans>
              </LoadingButton>
            </div>
          </div>
        ) : (
          <LoadingButton
            type='submit'
            loading={pending}
            loadingText={t`Importing`}
            disabled={!form.formState.isValid}
          >
            <Trans>Import</Trans>
          </LoadingButton>
        )}
      </form>
    </Form>
  );
}

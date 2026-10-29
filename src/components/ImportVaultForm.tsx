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
import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Textarea } from '@/components/ui/textarea';
import { useWallet } from '@/contexts/WalletContext';
import { useErrors } from '@/hooks/useErrors';
import { fetchState } from '@/state';
import { zodResolver } from '@hookform/resolvers/zod';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { useRef, useState } from 'react';
import { useForm } from 'react-hook-form';
import { useNavigate } from 'react-router-dom';
import * as z from 'zod';

export type VaultSource = 'launcher_id' | 'address' | 'config';

interface ImportVaultFormProps {
  /** A launcher id to start with, such as one found from a watched address. */
  launcherId?: string;
}

/** Imports an existing vault as a read-only wallet, from its launcher id, an
 *  address it has spent from, or a config file. */
export function ImportVaultForm({ launcherId }: ImportVaultFormProps) {
  const navigate = useNavigate();
  const { addError } = useErrors();
  const { setWallet } = useWallet();
  const [pending, setPending] = useState(false);
  const [source, setSource] = useState<VaultSource>('launcher_id');
  const fileRef = useRef<HTMLInputElement>(null);

  const formSchema = z.object({
    name: z.string().min(1),
    value: z.string().trim().min(1),
    emoji: z.string().nullable().optional(),
  });

  const form = useForm<z.infer<typeof formSchema>>({
    resolver: zodResolver(formSchema),
    defaultValues: { name: '', value: launcherId ?? '' },
    mode: 'onChange',
  });

  const changeSource = (value: string) => {
    setSource(value as VaultSource);
    form.setValue('value', '', { shouldValidate: true });
  };

  const submit = (values: z.infer<typeof formSchema>) => {
    setPending(true);

    const value = values.value.trim();

    commands
      .importVault({
        name: values.name,
        launcher_id: source === 'launcher_id' ? value : null,
        address: source === 'address' ? value : null,
        config: source === 'config' ? value : null,
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

        <Tabs value={source} onValueChange={changeSource}>
          <TabsList className='w-full'>
            <TabsTrigger value='launcher_id' className='flex-1'>
              <Trans>Launcher ID</Trans>
            </TabsTrigger>
            <TabsTrigger value='address' className='flex-1'>
              <Trans>Address</Trans>
            </TabsTrigger>
            <TabsTrigger value='config' className='flex-1'>
              <Trans>Config File</Trans>
            </TabsTrigger>
          </TabsList>
        </Tabs>

        <input
          ref={fileRef}
          type='file'
          accept='application/json,.json'
          className='hidden'
          onChange={async (event) => {
            const file = event.target.files?.[0];
            event.target.value = '';
            if (!file) return;
            form.setValue('value', await file.text(), {
              shouldValidate: true,
            });
          }}
        />

        <FormField
          control={form.control}
          name='value'
          render={({ field }) => (
            <FormItem>
              <FormLabel>
                {source === 'launcher_id' ? (
                  <Trans>Launcher ID</Trans>
                ) : source === 'address' ? (
                  <Trans>Vault Address</Trans>
                ) : (
                  <Trans>Config File</Trans>
                )}
              </FormLabel>
              <FormControl>
                {source === 'config' ? (
                  <Textarea className='resize-none h-28 font-mono' {...field} />
                ) : (
                  <Input required {...field} />
                )}
              </FormControl>
              <FormDescription>
                {source === 'launcher_id' ? (
                  <Trans>
                    The vault&apos;s launcher ID. Its config is read from the
                    memos on chain, if it has them.
                  </Trans>
                ) : source === 'address' ? (
                  <Trans>
                    An address the vault has already spent from. An address that
                    has only received coins doesn&apos;t reveal its vault yet.
                  </Trans>
                ) : (
                  <Trans>
                    A config file exported from Sage or Chia Cloud Wallet. It
                    must match the vault on chain.
                  </Trans>
                )}
              </FormDescription>
              <FormMessage />
            </FormItem>
          )}
        />

        {source === 'config' && (
          <button
            type='button'
            className='text-sm underline text-muted-foreground'
            onClick={() => fileRef.current?.click()}
          >
            <Trans>Choose a file</Trans>
          </button>
        )}

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

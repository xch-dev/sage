import Header from '@/components/Header';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
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
import { Label } from '@/components/ui/label';
import { Switch } from '@/components/ui/switch';
import { useErrors } from '@/hooks/useErrors';
import { zodResolver } from '@hookform/resolvers/zod';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import { CopyIcon, RefreshCwIcon } from 'lucide-react';
import { useCallback, useEffect, useState } from 'react';
import { useForm } from 'react-hook-form';
import { useNavigate } from 'react-router-dom';
import * as z from 'zod';
import { commands } from '../bindings';
import Container from '../components/Container';
import { fetchState } from '../state';
import SafeAreaView from '@/components/SafeAreaView';
import { Trans } from '@lingui/react/macro';
import { t } from '@lingui/core/macro';

export default function CreateWallet() {
  const { addError } = useErrors();
  const navigate = useNavigate();

  const submit = (values: z.infer<typeof formSchema>) => {
    commands
      .importKey({
        name: values.walletName,
        key: values.mnemonic,
        save_secrets: values.saveMnemonic,
      })
      .catch(addError)
      .then(fetchState)
      .then(() => navigate('/wallet'));
  };

  return (
    <SafeAreaView>
      <Header title={t`Create Wallet`} back={() => navigate('/')} />
      <Container>
        <CreateForm onSubmit={submit} />
      </Container>
    </SafeAreaView>
  );
}

const formSchema = z.object({
  walletName: z.string(),
  mnemonic: z.string(),
  use24Words: z.boolean(),
  saveMnemonic: z.boolean(),
});

function CreateForm(props: {
  onSubmit: (values: z.infer<typeof formSchema>) => void;
}) {
  const { addError } = useErrors();

  const form = useForm<z.infer<typeof formSchema>>({
    resolver: zodResolver(formSchema),
  });

  const use24Words = form.watch('use24Words', true);

  const loadMnemonic = useCallback(() => {
    commands
      .generateMnemonic({ use_24_words: use24Words })
      .then((data) => {
        form.setValue('mnemonic', data.mnemonic);
      })
      .catch(addError);
  }, [form, use24Words, addError]);

  useEffect(() => {
    loadMnemonic();
  }, [loadMnemonic]);

  const mnemonic = form.watch('mnemonic');
  const copyMnemonic = useCallback(() => {
    if (!mnemonic) return;
    writeText(mnemonic);
  }, [mnemonic]);

  const [isConfirmOpen, setConfirmOpen] = useState(false);

  const confirmAndSubmit = (values: z.infer<typeof formSchema>) => {
    if (!values.saveMnemonic) {
      setConfirmOpen(true);
    } else {
      props.onSubmit(values);
    }
  };

  return (
    <Form {...form}>
      <form
        onSubmit={form.handleSubmit(confirmAndSubmit)}
        className='space-y-4 max-w-xl mx-auto py-0'
      >
        <FormField
          control={form.control}
          name='walletName'
          render={({ field }) => (
            <FormItem>
              <FormLabel>
                <Trans>Wallet Name</Trans>
              </FormLabel>
              <FormControl>
                <Input placeholder='' required {...field} />
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />

        <FormField
          control={form.control}
          name='use24Words'
          defaultValue={true}
          render={({ field }) => (
            <FormItem className='flex flex-row items-center justify-between rounded-lg border p-4 gap-2'>
              <label htmlFor='use24Words' className='space-y-0.5'>
                <FormLabel>
                  <Trans>Use 24 words</Trans>
                </FormLabel>
                <FormDescription>
                  <Trans>
                    While 12 word mnemonics are sufficiently hard to crack, you
                    can choose to use 24 instead to increase security.
                  </Trans>
                </FormDescription>
              </label>
              <FormControl>
                <Switch
                  id='use24Words'
                  checked={field.value}
                  onCheckedChange={field.onChange}
                />
              </FormControl>
            </FormItem>
          )}
        />

        <FormField
          control={form.control}
          name='saveMnemonic'
          defaultValue={true}
          render={({ field }) => (
            <FormItem className='flex flex-row items-center justify-between rounded-lg border p-4 gap-2'>
              <label htmlFor='saveMnemonic' className='space-y-0.5'>
                <FormLabel>
                  <Trans>Save mnemonic</Trans>
                </FormLabel>
                <FormDescription>
                  <Trans>
                    By disabling this you are creating a cold wallet, with no
                    ability to sign transactions. The mnemonic will need to be
                    saved elsewhere.
                  </Trans>
                </FormDescription>
              </label>
              <FormControl>
                <Switch
                  id='saveMnemonic'
                  checked={field.value}
                  onCheckedChange={field.onChange}
                />
              </FormControl>
            </FormItem>
          )}
        />

        <div className='mt-3'>
          <div className='flex justify-between items-center mb-2'>
            <Label>
              <Trans>Mnemonic</Trans>
            </Label>
            <div>
              <Button
                type='button'
                variant='ghost'
                size='sm'
                onClick={loadMnemonic}
              >
                <RefreshCwIcon className='h-4 w-4' />
              </Button>
              <Button
                type='button'
                variant='ghost'
                size='sm'
                onClick={copyMnemonic}
              >
                <CopyIcon className='h-4 w-4' />
              </Button>
            </div>
          </div>
          <div className='flex flex-wrap'>
            {form
              .watch('mnemonic')
              ?.split(' ')
              .map((word, i) => (
                <Badge
                  key={i}
                  variant='outline'
                  className='py-1.5 px-2.5 m-0.5 rounded-lg font-medium'
                >
                  {word}
                </Badge>
              ))}
          </div>
        </div>

        <Button type='submit'>
          <Trans>Submit</Trans>
        </Button>
      </form>
      <Dialog open={isConfirmOpen} onOpenChange={setConfirmOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>
              <Trans>Did you save your mnemonic?</Trans>
            </DialogTitle>
            <DialogDescription>
              <Trans>
                Make sure you have saved your mnemonic. You will not be able to
                access it later, since it will not be saved in the wallet. You
                will also not be able to make transactions with this wallet.
              </Trans>
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant='outline' onClick={() => setConfirmOpen(false)}>
              <Trans>Cancel</Trans>
            </Button>
            <Button
              onClick={() => {
                setConfirmOpen(false);
                props.onSubmit(form.getValues());
              }}
            >
              <Trans>Confirm</Trans>
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </Form>
  );
}

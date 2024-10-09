import { Alert } from '@mui/material';
import { useCallback, useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { commands } from '../bindings';
import Container from '../components/Container';
import { fetchState } from '../state';
import Header from '@/components/Header';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import * as z from 'zod';
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
import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Separator } from '@/components/ui/separator';
import { Badge } from '@/components/ui/badge';
import { Label } from '@/components/ui/label';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import { CopyIcon, RefreshCwIcon } from 'lucide-react';

export default function CreateWallet() {
  const navigate = useNavigate();

  const [error, setError] = useState<string | null>(null);

  const submit = (values: z.infer<typeof formSchema>) => {
    commands
      .createWallet(values.walletName, values.mnemonic, values.saveMnemonic)
      .then((res) => {
        if (res.status === 'ok') {
          fetchState().then(() => {
            navigate('/wallet');
          });
        }
      })
      .catch(setError);
  };

  return (
    <>
      <Header title='Create Wallet' back={() => navigate('/')} />
      <Container>
        <CreateForm onSubmit={submit} />

        {error && (
          <Alert variant='outlined' severity='error' sx={{ mt: 2 }}>
            {error}
          </Alert>
        )}
      </Container>
    </>
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
  const form = useForm<z.infer<typeof formSchema>>({
    resolver: zodResolver(formSchema),
  });

  const use24Words = form.watch('use24Words', true);

  const loadMnemonic = useCallback(() => {
    commands.generateMnemonic(use24Words).then((res) => {
      if (res.status === 'ok') {
        form.setValue('mnemonic', res.data);
      }
    });
  }, [form, use24Words]);

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
        className='space-y-4 max-w-xl mx-auto py-4'
      >
        <FormField
          control={form.control}
          name='walletName'
          render={({ field }) => (
            <FormItem>
              <FormLabel>Wallet Name</FormLabel>
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
                <FormLabel>Use 24 words</FormLabel>
                <FormDescription>
                  While 12 word mnemonics are sufficiently hard to crack, you
                  can choose to use 24 instead to increase security.
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
                <FormLabel>Save mnemonic</FormLabel>
                <FormDescription>
                  By disabling this you are creating a cold wallet, with no
                  ability to sign transactions. The mnemonic will need to be
                  saved elsewhere.
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

        <Separator className='my-4' />

        <div className='mt-3'>
          <div className='flex justify-between items-center mb-2'>
            <Label>Mnemonic</Label>
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
                <Badge key={i} variant='outline' className='m-0.5 font-medium'>
                  {word}
                </Badge>
              ))}
          </div>
        </div>

        <Button type='submit'>Submit</Button>
      </form>
      <Dialog open={isConfirmOpen} onOpenChange={setConfirmOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Did you save your mnemonic?</DialogTitle>
            <DialogDescription>
              Make sure you have saved your mnemonic. You will not be able to
              access it later, since it will not be saved in the wallet. You
              will also not be able to make transactions with this wallet.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant='outline' onClick={() => setConfirmOpen(false)}>
              Cancel
            </Button>
            <Button
              onClick={() => {
                setConfirmOpen(false);
                props.onSubmit(form.getValues());
              }}
            >
              Confirm
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </Form>
  );
}

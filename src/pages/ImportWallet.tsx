import Header from '@/components/Header';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
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
import { Textarea } from '@/components/ui/textarea';
import { zodResolver } from '@hookform/resolvers/zod';
import { AlertCircle } from 'lucide-react';
import { useState } from 'react';
import { useForm } from 'react-hook-form';
import { useNavigate } from 'react-router-dom';
import * as z from 'zod';
import { commands } from '../bindings';
import Container from '../components/Container';
import { fetchState } from '../state';

export default function ImportWallet() {
  const navigate = useNavigate();

  const [error, setError] = useState<string | null>(null);

  const submit = (values: z.infer<typeof formSchema>) => {
    commands
      .importKey({ name: values.walletName, key: values.walletKey })
      .then((res) => {
        if (res.status === 'ok') {
          fetchState().then(() => {
            navigate('/wallet');
          });
        } else {
          setError(res.error.reason);
        }
      });
  };

  return (
    <>
      <Header title='Import Wallet' back={() => navigate('/')} />

      <Container>
        <ImportForm onSubmit={submit} />

        {error && (
          <Alert variant='destructive'>
            <AlertCircle className='h-4 w-4' />
            <AlertTitle>Error</AlertTitle>
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        )}
      </Container>
    </>
  );
}

const formSchema = z.object({
  walletName: z.string(),
  walletKey: z.string(),
});

function ImportForm(props: {
  onSubmit: (values: z.infer<typeof formSchema>) => void;
}) {
  // Insert constants here
  const form = useForm<z.infer<typeof formSchema>>({
    resolver: zodResolver(formSchema),
  });

  return (
    <Form {...form}>
      <form
        onSubmit={form.handleSubmit(props.onSubmit)}
        className='space-y-4 max-w-xl mx-auto py-4'
      >
        <FormField
          control={form.control}
          name='walletName'
          render={({ field }) => (
            <FormItem>
              <FormLabel>Wallet Name</FormLabel>
              <FormControl>
                <Input required {...field} />
              </FormControl>

              <FormMessage />
            </FormItem>
          )}
        />

        <FormField
          control={form.control}
          name='walletKey'
          render={({ field }) => (
            <FormItem>
              <FormLabel>Wallet Key</FormLabel>
              <FormControl>
                <Textarea className='resize-none' {...field} />
              </FormControl>
              <FormDescription>
                Enter your mnemonic, private key, or public key below. If it's a
                public key, it will be imported as a read-only cold wallet.
              </FormDescription>
              <FormMessage />
            </FormItem>
          )}
        />
        <Button type='submit'>Import Wallet</Button>
      </form>
    </Form>
  );
}

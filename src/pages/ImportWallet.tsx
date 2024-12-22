import Header from '@/components/Header';
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
import { useErrors } from '@/hooks/useErrors';
import { zodResolver } from '@hookform/resolvers/zod';
import { useForm } from 'react-hook-form';
import { useNavigate } from 'react-router-dom';
import * as z from 'zod';
import { commands } from '../bindings';
import Container from '../components/Container';
import { fetchState } from '../state';
import SafeAreaView from '@/components/SafeAreaView';
import { Trans } from '@lingui/react/macro';
import { t } from '@lingui/core/macro';

export default function ImportWallet() {
  const navigate = useNavigate();
  const { addError } = useErrors();

  const submit = (values: z.infer<typeof formSchema>) => {
    commands
      .importKey({ name: values.walletName, key: values.walletKey })
      .then(fetchState)
      .then(() => navigate('/wallet'))
      .catch(addError);
  };

  return (
    <SafeAreaView>
      <Header title={t`Import Wallet`} back={() => navigate('/')} />
      <Container>
        <ImportForm onSubmit={submit} />
      </Container>
    </SafeAreaView>
  );
}

const formSchema = z.object({
  walletName: z.string(),
  walletKey: z.string(),
});

function ImportForm(props: {
  onSubmit: (values: z.infer<typeof formSchema>) => void;
}) {
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
          name='walletKey'
          render={({ field }) => (
            <FormItem>
              <FormLabel>
                <Trans>Wallet Key</Trans>
              </FormLabel>
              <FormControl>
                <Textarea className='resize-none' {...field} />
              </FormControl>
              <FormDescription>
                <Trans>
                  Enter your mnemonic, private key, or public key below. If it's
                  a public key, it will be imported as a read-only cold wallet.
                </Trans>
              </FormDescription>
              <FormMessage />
            </FormItem>
          )}
        />
        <Button type='submit'>
          <Trans>Import Wallet</Trans>
        </Button>
      </form>
    </Form>
  );
}

import ConfirmationDialog from '@/components/ConfirmationDialog';
import { TokenConfirmation } from '@/components/confirmations/TokenConfirmation';
import Header from '@/components/Header';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Button } from '@/components/ui/button';
import {
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from '@/components/ui/form';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { FeeAmountInput, TokenAmountInput } from '@/components/ui/masked-input';
import { Switch } from '@/components/ui/switch';
import { useErrors } from '@/hooks/useErrors';
import { amount, positiveAmount } from '@/lib/formTypes';
import { toMojos } from '@/lib/utils';
import { zodResolver } from '@hookform/resolvers/zod';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { TriangleAlertIcon } from 'lucide-react';
import { useState } from 'react';
import { useForm } from 'react-hook-form';
import { useNavigate } from 'react-router-dom';
import * as z from 'zod';
import { commands, TransactionResponse } from '../bindings';
import Container from '../components/Container';
import { useWalletState } from '../state';

export default function IssueToken() {
  const navigate = useNavigate();
  const walletState = useWalletState();
  const { addError } = useErrors();
  const [response, setResponse] = useState<TransactionResponse | null>(null);

  const formSchema = z.object({
    name: z.string().min(1, t`Name is required`),
    ticker: z.string().min(1, t`Ticker is required`),
    amount: positiveAmount(3),
    fee: amount(walletState.sync.unit.precision).optional(),
    revocable: z.boolean(),
  });

  const form = useForm<z.infer<typeof formSchema>>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      revocable: false,
    },
  });

  const onSubmit = (values: z.infer<typeof formSchema>) => {
    commands
      .issueCat({
        name: values.name,
        ticker: values.ticker,
        amount: toMojos(values.amount.toString(), 3),
        revocable: values.revocable,
        fee: toMojos(
          values.fee?.toString() || '0',
          walletState.sync.unit.precision,
        ),
      })
      .then(setResponse)
      .catch(addError);
  };

  return (
    <>
      <Header title={t`Issue Token`} />

      <Container className='max-w-xl'>
        <Form {...form}>
          <form onSubmit={form.handleSubmit(onSubmit)} className='space-y-4'>
            <div className='grid sm:grid-cols-2 gap-4'>
              <FormField
                control={form.control}
                name='name'
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>
                      <Trans>Name</Trans>
                    </FormLabel>
                    <FormControl>
                      <Input placeholder={t`Display name`} {...field} />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name='ticker'
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>
                      <Trans>Ticker</Trans>
                    </FormLabel>
                    <FormControl>
                      <Input placeholder={t`Currency Symbol`} {...field} />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
            </div>

            <div className='grid sm:grid-cols-2 gap-4'>
              <FormField
                control={form.control}
                name='amount'
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>
                      <Trans>Amount</Trans>
                    </FormLabel>
                    <FormControl>
                      <div className='relative'>
                        <TokenAmountInput
                          {...field}
                          className='pr-12'
                          precision={3}
                          hideMaxButton
                        />
                        <div className='pointer-events-none absolute inset-y-0 right-0 flex items-center pr-3'>
                          <span className='text-muted-foreground text-sm'>
                            CAT
                          </span>
                        </div>
                      </div>
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name='fee'
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>
                      <Trans>Network Fee</Trans>
                    </FormLabel>
                    <FormControl>
                      <div className='relative'>
                        <FeeAmountInput {...field} className='pr-12' />
                      </div>
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
            </div>

            <FormField
              control={form.control}
              name='revocable'
              render={({ field }) => (
                <FormItem className='flex items-center justify-between gap-4 rounded-lg border p-4'>
                  <div className='space-y-1'>
                    <Label htmlFor='revocable'>
                      <Trans>Revocable CAT</Trans>
                    </Label>
                    <p className='text-sm text-muted-foreground'>
                      <Trans>
                        Use this wallet&apos;s change address as the
                        token&apos;s revocation address.
                      </Trans>
                    </p>
                  </div>
                  <FormControl>
                    <Switch
                      id='revocable'
                      checked={field.value}
                      onCheckedChange={field.onChange}
                    />
                  </FormControl>
                </FormItem>
              )}
            />

            {form.watch('revocable') && (
              <Alert variant='warning'>
                <TriangleAlertIcon className='h-4 w-4' aria-hidden='true' />
                <AlertTitle>
                  <Trans>Revocable CAT Warning</Trans>
                </AlertTitle>
                <AlertDescription>
                  <Trans>
                    Only issue a revocable CAT if you understand the risks. Sage
                    cannot revoke it; revocation currently requires external
                    tools.
                  </Trans>
                </AlertDescription>
              </Alert>
            )}

            <Button type='submit'>
              <Trans>Issue Token</Trans>
            </Button>
          </form>
        </Form>
      </Container>

      <ConfirmationDialog
        response={response}
        close={() => setResponse(null)}
        onConfirm={() => navigate('/wallet')}
        showRecipientDetails={false}
        additionalData={
          form.getValues().name &&
          form.getValues().ticker &&
          form.getValues().amount
            ? {
                title: t`Token Details`,
                content: (
                  <TokenConfirmation
                    type='issue'
                    name={form.getValues().name}
                    ticker={form.getValues().ticker}
                    amount={form.getValues().amount.toString()}
                    revocable={form.getValues().revocable}
                  />
                ),
              }
            : undefined
        }
      />
    </>
  );
}

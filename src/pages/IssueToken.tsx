import ConfirmationDialog from '@/components/ConfirmationDialog';
import Header from '@/components/Header';
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
import { useErrors } from '@/hooks/useErrors';
import { amount, positiveAmount } from '@/lib/formTypes';
import { toMojos } from '@/lib/utils';
import { zodResolver } from '@hookform/resolvers/zod';
import { useState } from 'react';
import { useForm } from 'react-hook-form';
import { useNavigate } from 'react-router-dom';
import * as z from 'zod';
import { commands, TransactionResponse } from '../bindings';
import Container from '../components/Container';
import { useWalletState } from '../state';
import { TokenAmountInput } from '@/components/ui/masked-input';
import { Trans } from '@lingui/react/macro';
import { t } from '@lingui/core/macro';
import { TokenConfirmation } from '@/components/confirmations/TokenConfirmation';

export default function IssueToken() {
  const navigate = useNavigate();
  const walletState = useWalletState();
  const { addError } = useErrors();
  const [response, setResponse] = useState<TransactionResponse | null>(null);

  const formSchema = z.object({
    name: z.string().min(1, t`Name is required`),
    ticker: z.string().min(1, t`Ticker is required`),
    amount: positiveAmount(3),
    fee: amount(walletState.sync.unit.decimals).optional(),
  });

  const form = useForm<z.infer<typeof formSchema>>({
    resolver: zodResolver(formSchema),
  });

  const onSubmit = (values: z.infer<typeof formSchema>) => {
    commands
      .issueCat({
        name: values.name,
        ticker: values.ticker,
        amount: toMojos(values.amount.toString(), 3),
        fee: toMojos(
          values.fee?.toString() || '0',
          walletState.sync.unit.decimals,
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
                        <TokenAmountInput {...field} className='pr-12' />
                        <div className='pointer-events-none absolute inset-y-0 right-0 flex items-center pr-3'>
                          <span className='text-gray-500 text-sm'>CAT</span>
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
                      <Trans>Fee</Trans>
                    </FormLabel>
                    <FormControl>
                      <div className='relative'>
                        <TokenAmountInput {...field} className='pr-12' />
                        <div className='pointer-events-none absolute inset-y-0 right-0 flex items-center pr-3'>
                          <span className='text-gray-500 text-sm'>
                            {walletState.sync.unit.ticker}
                          </span>
                        </div>
                      </div>
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
            </div>

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
                  />
                ),
              }
            : undefined
        }
      />
    </>
  );
}

import ConfirmationDialog from '@/components/ConfirmationDialog';
import { TokenConfirmation } from '@/components/confirmations/TokenConfirmation';
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
import {
  FeeAmountInput,
  IntegerInput,
  TokenAmountInput,
} from '@/components/ui/masked-input';
import { Switch } from '@/components/ui/switch';
import { useErrors } from '@/hooks/useErrors';
import { amount, positiveAmount } from '@/lib/formTypes';
import { toMojos } from '@/lib/utils';
import { zodResolver } from '@hookform/resolvers/zod';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
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
    enableFeeLayer: z.boolean(),
    feeRecipient: z.string().optional(),
    feeBasisPoints: z.string().optional(),
    feeMinFee: amount(3).optional(),
    allowZeroPrice: z.boolean(),
    allowRevokeFeeBypass: z.boolean(),
  });

  const form = useForm<z.infer<typeof formSchema>>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      enableFeeLayer: false,
      allowZeroPrice: false,
      allowRevokeFeeBypass: false,
    },
  });

  const onSubmit = async (values: z.infer<typeof formSchema>) => {
    let feePolicy: {
      recipient: string;
      fee_basis_points: number;
      min_fee: string;
      allow_zero_price: boolean;
      allow_revoke_fee_bypass: boolean;
    } | null = null;

    if (values.enableFeeLayer) {
      const recipient = values.feeRecipient?.trim() ?? '';
      const feeBasisPoints = Number.parseInt(values.feeBasisPoints || '0', 10);

      if (!recipient) {
        addError({
          kind: 'invalid',
          reason: t`Fee recipient is required when fee layer is enabled.`,
        });
        return;
      }

      let isAddressValid = false;
      try {
        isAddressValid = await commands.validateAddress(recipient);
      } catch (error) {
        addError({
          kind: 'internal',
          reason: `${error}`,
        });
        return;
      }
      if (!isAddressValid) {
        addError({
          kind: 'invalid',
          reason: t`Fee recipient address is invalid for the current network.`,
        });
        return;
      }

      if (
        Number.isNaN(feeBasisPoints) ||
        feeBasisPoints < 0 ||
        feeBasisPoints > 65535
      ) {
        addError({
          kind: 'invalid',
          reason: t`Fee basis points must be between 0 and 65535.`,
        });
        return;
      }

      feePolicy = {
        recipient,
        fee_basis_points: feeBasisPoints,
        min_fee: toMojos(values.feeMinFee?.toString() || '0', 3),
        allow_zero_price: values.allowZeroPrice,
        allow_revoke_fee_bypass: values.allowRevokeFeeBypass,
      };
    }

    commands
      .issueCat({
        name: values.name,
        ticker: values.ticker,
        amount: toMojos(values.amount.toString(), 3),
        fee: toMojos(
          values.fee?.toString() || '0',
          walletState.sync.unit.precision,
        ),
        fee_policy: feePolicy ?? undefined,
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

            <div className='rounded-md border p-4 space-y-4'>
              <div className='flex items-center gap-2'>
                <Switch
                  checked={form.watch('enableFeeLayer')}
                  onCheckedChange={(checked) =>
                    form.setValue('enableFeeLayer', checked)
                  }
                />
                <FormLabel>
                  <Trans>Add fee layer</Trans>
                </FormLabel>
              </div>

              {form.watch('enableFeeLayer') && (
                <>
                  <FormField
                    control={form.control}
                    name='feeRecipient'
                    render={({ field }) => (
                      <FormItem>
                        <FormLabel>
                          <Trans>Fee Recipient</Trans>
                        </FormLabel>
                        <FormControl>
                          <Input
                            {...field}
                            autoCorrect='off'
                            autoCapitalize='off'
                            autoComplete='off'
                            placeholder={t`Enter recipient address`}
                          />
                        </FormControl>
                        <FormMessage />
                      </FormItem>
                    )}
                  />

                  <div className='grid sm:grid-cols-2 gap-4'>
                    <FormField
                      control={form.control}
                      name='feeBasisPoints'
                      render={({ field }) => (
                        <FormItem>
                          <FormLabel>
                            <Trans>Fee (basis points)</Trans>
                          </FormLabel>
                          <FormControl>
                            <IntegerInput
                              value={field.value}
                              min={0}
                              max={65535}
                              placeholder='0'
                              onValueChange={(values) =>
                                field.onChange(values.value)
                              }
                            />
                          </FormControl>
                          <FormMessage />
                        </FormItem>
                      )}
                    />

                    <FormField
                      control={form.control}
                      name='feeMinFee'
                      render={({ field }) => (
                        <FormItem>
                          <FormLabel>
                            <Trans>Minimum Fee</Trans>
                          </FormLabel>
                          <FormControl>
                            <TokenAmountInput
                              {...field}
                              precision={3}
                              hideMaxButton
                              placeholder={t`Minimum CAT fee`}
                            />
                          </FormControl>
                          <FormMessage />
                        </FormItem>
                      )}
                    />
                  </div>

                  <FormField
                    control={form.control}
                    name='allowZeroPrice'
                    render={({ field }) => (
                      <FormItem className='flex items-center justify-between rounded-md border px-3 py-2'>
                        <FormLabel>
                          <Trans>Allow zero-price transfers</Trans>
                        </FormLabel>
                        <FormControl>
                          <Switch
                            checked={field.value}
                            onCheckedChange={field.onChange}
                          />
                        </FormControl>
                      </FormItem>
                    )}
                  />

                  <FormField
                    control={form.control}
                    name='allowRevokeFeeBypass'
                    render={({ field }) => (
                      <FormItem className='flex items-center justify-between rounded-md border px-3 py-2'>
                        <FormLabel>
                          <Trans>Allow revoke fee bypass</Trans>
                        </FormLabel>
                        <FormControl>
                          <Switch
                            checked={field.value}
                            onCheckedChange={field.onChange}
                          />
                        </FormControl>
                      </FormItem>
                    )}
                  />
                </>
              )}
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
                    feePolicy={
                      form.getValues().enableFeeLayer
                        ? {
                            recipient: form.getValues().feeRecipient || '',
                            feeBasisPoints: form.getValues().feeBasisPoints || '0',
                            minFee: form.getValues().feeMinFee || '0',
                            allowZeroPrice: form.getValues().allowZeroPrice,
                            allowRevokeFeeBypass:
                              form.getValues().allowRevokeFeeBypass,
                          }
                        : undefined
                    }
                  />
                ),
              }
            : undefined
        }
      />
    </>
  );
}

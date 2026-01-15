import ConfirmationDialog from '@/components/ConfirmationDialog';
import { TokenConfirmation } from '@/components/confirmations/TokenConfirmation';
import Container from '@/components/Container';
import Header from '@/components/Header';
import { NumberFormat } from '@/components/NumberFormat';
import { PasteInput } from '@/components/PasteInput';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Button } from '@/components/ui/button';
import { Card, CardContent } from '@/components/ui/card';
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
import {
  FeeAmountInput,
  IntegerInput,
  TokenAmountInput,
} from '@/components/ui/masked-input';
import { Switch } from '@/components/ui/switch';
import { Textarea } from '@/components/ui/textarea';
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/components/ui/tooltip';
import { useDefaultClawback } from '@/hooks/useDefaultClawback';
import { useErrors } from '@/hooks/useErrors';
import { useScannerOrClipboard } from '@/hooks/useScannerOrClipboard';
import { amount, positiveAmount } from '@/lib/formTypes';
import { fromMojos, toDecimal, toHex, toMojos } from '@/lib/utils';
import { useWalletState } from '@/state';
import { zodResolver } from '@hookform/resolvers/zod';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import BigNumber from 'bignumber.js';
import { AlertCircleIcon, ArrowUpToLine } from 'lucide-react';
import { useCallback, useEffect, useState } from 'react';
import { useForm } from 'react-hook-form';
import { useNavigate, useParams } from 'react-router-dom';
import * as z from 'zod';
import {
  commands,
  events,
  TokenRecord,
  TransactionResponse,
} from '../bindings';

function stringToUint8Array(str: string): Uint8Array {
  return new TextEncoder().encode(str);
}

export default function Send() {
  let { asset_id: assetId = null } = useParams();
  if (assetId === 'xch') assetId = null;

  const { addError } = useErrors();
  const { clawback } = useDefaultClawback();

  const navigate = useNavigate();
  const walletState = useWalletState();

  const [asset, setAsset] = useState<TokenRecord | null>(null);
  const [response, setResponse] = useState<TransactionResponse | null>(null);
  const [currentMemo, setCurrentMemo] = useState<string | undefined>(undefined);

  const [bulk, setBulk] = useState(false);

  const ticker = asset?.ticker || 'CAT';

  const updateToken = useCallback(
    () =>
      commands
        .getToken({ asset_id: assetId })
        .then((data) => setAsset(data.token))
        .catch(addError),
    [assetId, addError],
  );

  useEffect(() => {
    updateToken();

    const unlisten = events.syncEvent.listen((event) => {
      const type = event.payload.type;
      if (
        type === 'coin_state' ||
        type === 'puzzle_batch_synced' ||
        type === 'cat_info'
      ) {
        updateToken();
      }
    });

    return () => {
      unlisten.then((u) => u());
    };
  }, [updateToken]);

  const addressList = (value: string) => {
    if (bulk) {
      return value
        .trim()
        .split(/\s*,\s*|\s+/)
        .map((line) => line.trim())
        .filter(Boolean);
    } else {
      return [value.trim()];
    }
  };

  const formSchema = z.object({
    address: z
      .string()
      .refine(
        (address) =>
          Promise.all(
            addressList(address).map((address) =>
              commands.validateAddress(address).catch(addError),
            ),
          ).then((values) => values.every(Boolean)),
        bulk ? t`Invalid addresses` : t`Invalid address`,
      ),
    amount: positiveAmount(asset?.precision || 12).refine(
      (amount) =>
        asset
          ? BigNumber(amount).lte(toDecimal(asset.balance, asset.precision))
          : true,
      'Amount exceeds balance',
    ),
    fee: amount(walletState.sync.unit.precision).optional(),
    memo: z.string().optional(),
    clawbackEnabled: z.boolean().optional(),
    clawback: z
      .object({
        days: z.string(),
        hours: z.string(),
        minutes: z.string(),
      })
      .optional(),
  });

  const form = useForm<z.infer<typeof formSchema>>({
    resolver: zodResolver(formSchema),
  });

  const { handleScanOrPaste } = useScannerOrClipboard((scanResValue) => {
    form.setValue('address', scanResValue);
  });

  const onSubmit = () => {
    const values = form.getValues();
    const memos = values.memo ? [toHex(stringToUint8Array(values.memo))] : [];

    // Store the memo for the confirmation dialog
    setCurrentMemo(values.memo);

    // Calculate clawback seconds if enabled
    let clawback: number | null = null;

    if (values.clawbackEnabled && values.clawback) {
      const days = parseInt(values.clawback.days) || 0;
      const hours = parseInt(values.clawback.hours) || 0;
      const minutes = parseInt(values.clawback.minutes) || 0;
      clawback =
        Math.ceil(Date.now() / 1000) +
        (days * 24 * 60 * 60 + hours * 60 * 60 + minutes * 60);
    }

    let result: Promise<TransactionResponse>;

    const amount = toMojos(values.amount.toString(), asset?.precision || 12);
    const fee = toMojos(
      values.fee?.toString() || '0',
      walletState.sync.unit.precision,
    );

    if (!assetId) {
      if (bulk) {
        result = commands.bulkSendXch({
          addresses: [...new Set(addressList(values.address))],
          amount,
          fee,
          memos,
        });
      } else {
        result = commands.sendXch({
          address: values.address,
          amount,
          fee,
          memos,
          clawback,
        });
      }
    } else {
      if (bulk) {
        result = commands.bulkSendCat({
          asset_id: assetId ?? '',
          addresses: [...new Set(addressList(values.address))],
          amount,
          fee,
          memos,
        });
      } else {
        result = commands.sendCat({
          asset_id: assetId ?? '',
          address: values.address,
          amount,
          fee,
          memos,
          clawback,
        });
      }
    }

    result.then((confirmation) => setResponse(confirmation)).catch(addError);
  };

  return (
    <>
      <Header title={t`Send ${ticker}`} back={() => navigate(-1)} />

      <Container className='max-w-xl'>
        {asset && (
          <Card className='mb-6'>
            <CardContent className='pt-6'>
              <div className='text-sm text-muted-foreground'>
                Available Balance
              </div>
              <div className='text-2xl font-medium mt-1'>
                <NumberFormat
                  value={fromMojos(asset.balance, asset.precision)}
                  minimumFractionDigits={0}
                  maximumFractionDigits={asset.precision}
                />{' '}
                {asset.ticker}
              </div>
            </CardContent>
          </Card>
        )}

        <div className='flex items-center gap-2 mb-4'>
          <Label htmlFor='bulk'>
            <Trans>Send in bulk (airdrop)</Trans>
          </Label>
          <Switch id='bulk' checked={bulk} onCheckedChange={setBulk} />
        </div>

        <Form {...form}>
          <form onSubmit={form.handleSubmit(onSubmit)} className='space-y-4'>
            <FormField
              control={form.control}
              name='address'
              render={({ field }) => (
                <FormItem>
                  <FormLabel>
                    <Trans>Address</Trans>
                  </FormLabel>
                  <FormControl>
                    {bulk ? (
                      <Textarea
                        autoCorrect='off'
                        autoCapitalize='off'
                        autoComplete='off'
                        placeholder={t`Enter multiple distinct addresses`}
                        {...field}
                      />
                    ) : (
                      <PasteInput
                        autoCorrect='off'
                        autoCapitalize='off'
                        autoComplete='off'
                        placeholder={t`Enter address`}
                        onEndIconClick={handleScanOrPaste}
                        {...field}
                      />
                    )}
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />

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
                      <div className='relative flex'>
                        <TokenAmountInput
                          {...field}
                          ticker={asset?.ticker}
                          precision={
                            asset?.precision ?? (assetId === null ? 12 : 3)
                          }
                          className='pr-12 !rounded-r-none z-10'
                        />
                        <TooltipProvider>
                          <Tooltip>
                            <TooltipTrigger asChild>
                              <Button
                                variant='outline'
                                size='icon'
                                type='button'
                                tabIndex={-1}
                                className='!border-l-0 !rounded-l-none flex-shrink-0'
                                onClick={() => {
                                  if (asset) {
                                    const maxAmount = fromMojos(
                                      asset.balance,
                                      asset.precision,
                                    );
                                    form.setValue(
                                      'amount',
                                      maxAmount.toString(),
                                      { shouldValidate: true },
                                    );
                                  }
                                }}
                              >
                                <ArrowUpToLine className='h-4 w-4' />
                              </Button>
                            </TooltipTrigger>
                            <TooltipContent>
                              <Trans>Use maximum balance</Trans>
                            </TooltipContent>
                          </Tooltip>
                        </TooltipProvider>
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

              <FormField
                control={form.control}
                name='memo'
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>
                      <Trans>Memo (optional)</Trans>
                    </FormLabel>
                    <FormControl>
                      <Input
                        autoCorrect='off'
                        autoCapitalize='off'
                        autoComplete='off'
                        placeholder={t`Enter memo`}
                        {...field}
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              {!bulk && (
                <div className='col-span-2'>
                  <div className='flex flex-col gap-2'>
                    <div className='flex items-center gap-2'>
                      <Label htmlFor='clawbackEnabled'>
                        <Trans>Enable clawback</Trans>
                      </Label>
                      <Switch
                        id='clawbackEnabled'
                        checked={form.watch('clawbackEnabled')}
                        onCheckedChange={(checked) => {
                          form.setValue('clawbackEnabled', checked);
                          if (checked) {
                            form.setValue('clawback', {
                              days: clawback?.days?.toString() ?? '0',
                              hours: clawback?.hours?.toString() ?? '0',
                              minutes: clawback?.minutes?.toString() ?? '0',
                            });
                          } else {
                            form.setValue('clawback', undefined);
                          }
                        }}
                      />
                    </div>

                    {form.watch('clawbackEnabled') && (
                      <div className='flex flex-col gap-4'>
                        <div className='flex gap-2'>
                          <div className='relative'>
                            <IntegerInput
                              className='pr-12'
                              value={form.watch('clawback.days')}
                              placeholder='0'
                              min={0}
                              onValueChange={(values: { value: string }) => {
                                form.setValue('clawback.days', values.value);
                              }}
                            />
                            <div className='pointer-events-none absolute inset-y-0 right-0 flex items-center pr-3'>
                              <span className='text-muted-foreground text-sm'>
                                <Trans>Days</Trans>
                              </span>
                            </div>
                          </div>

                          <div className='relative'>
                            <IntegerInput
                              className='pr-12'
                              value={form.watch('clawback.hours')}
                              placeholder='0'
                              min={0}
                              onValueChange={(values: { value: string }) => {
                                form.setValue('clawback.hours', values.value);
                              }}
                            />
                            <div className='pointer-events-none absolute inset-y-0 right-0 flex items-center pr-3'>
                              <span className='text-muted-foreground text-sm'>
                                <Trans>Hours</Trans>
                              </span>
                            </div>
                          </div>

                          <div className='relative'>
                            <IntegerInput
                              className='pr-12'
                              value={form.watch('clawback.minutes')}
                              placeholder='0'
                              min={0}
                              onValueChange={(values: { value: string }) => {
                                form.setValue('clawback.minutes', values.value);
                              }}
                            />
                            <div className='pointer-events-none absolute inset-y-0 right-0 flex items-center pr-3'>
                              <span className='text-muted-foreground text-sm'>
                                <Trans>Minutes</Trans>
                              </span>
                            </div>
                          </div>
                        </div>

                        <Alert variant='warning'>
                          <AlertCircleIcon className='h-4 w-4' />
                          <AlertTitle>
                            <Trans>Experimental Feature</Trans>
                          </AlertTitle>
                          <AlertDescription>
                            <Trans>
                              Support for Clawback v2 is limited at this time.
                              Please make sure the recipient supports it before
                              submitting this transaction.
                            </Trans>
                          </AlertDescription>
                        </Alert>
                      </div>
                    )}
                  </div>
                </div>
              )}
            </div>

            <Button type='submit'>
              <Trans>Send {ticker}</Trans>
            </Button>
          </form>
        </Form>
      </Container>

      <ConfirmationDialog
        response={response}
        close={() => setResponse(null)}
        onConfirm={() => navigate(-1)}
        additionalData={
          currentMemo
            ? {
                title: t`Send ${ticker}`,
                content: (
                  <TokenConfirmation type='send' currentMemo={currentMemo} />
                ),
              }
            : undefined
        }
      />
    </>
  );
}

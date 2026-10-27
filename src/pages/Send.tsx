import ConfirmationDialog from '@/components/ConfirmationDialog';
import {
  HandleResolution,
  HandleResolutionList,
} from '@/components/HandleResolution';
import { WalletAddressPicker } from '@/components/WalletAddressPicker';
import { TokenConfirmation } from '@/components/confirmations/TokenConfirmation';
import Container from '@/components/Container';
import { ExpirationInput } from '@/components/ExpirationInput';
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
import { FeeAmountInput, TokenAmountInput } from '@/components/ui/masked-input';
import { Switch } from '@/components/ui/switch';
import { Textarea } from '@/components/ui/textarea';
import { useDefaultClawback } from '@/hooks/useDefaultClawback';
import { useErrors } from '@/hooks/useErrors';
import { useScannerOrClipboard } from '@/hooks/useScannerOrClipboard';
import { expirationError, futureExpirationTimestamp } from '@/lib/expiration';
import { xchandlesEnabled } from '@/lib/features';
import { amount, positiveAmount } from '@/lib/formTypes';
import { validateRecipient, withHandleConfirmation } from '@/lib/handles';
import {
  fromHexStrict,
  fromMojos,
  isValidHexBytes,
  toDecimal,
  toHex,
  toMojos,
  utf8ToBytes,
} from '@/lib/utils';
import { useWalletState } from '@/state';
import { Memo, MemoMode } from '@/types/CoinMemo.ts';
import { zodResolver } from '@hookform/resolvers/zod';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import BigNumber from 'bignumber.js';
import { AlertCircleIcon } from 'lucide-react';
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

const emptyClawback = { days: '0', hours: '0', minutes: '0' };

export default function Send() {
  let { asset_id: assetId = null } = useParams();
  if (assetId === 'xch') assetId = null;

  const { addError } = useErrors();
  const { clawback } = useDefaultClawback();

  const navigate = useNavigate();
  const walletState = useWalletState();

  const [asset, setAsset] = useState<TokenRecord | null>(null);
  const [response, setResponse] = useState<TransactionResponse | null>(null);
  const [currentMemo, setCurrentMemo] = useState<Memo | undefined>(undefined);

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

  const formSchema = z
    .object({
      address: z
        .string()
        .refine(
          (address) =>
            Promise.all(
              addressList(address).map((address) =>
                validateRecipient(address).catch(addError),
              ),
            ).then((values) => values.every(Boolean)),
          xchandlesEnabled
            ? bulk
              ? t`Invalid addresses or handles`
              : t`Invalid address or handle`
            : bulk
              ? t`Invalid addresses`
              : t`Invalid address`,
        ),
      amount: positiveAmount(asset?.precision || 12).refine(
        (amount) =>
          asset
            ? BigNumber(amount).lte(
                toDecimal(asset.selectable_balance, asset.precision),
              )
            : true,
        'Amount exceeds spendable balance',
      ),
      fee: amount(walletState.sync.unit.precision).optional(),
      memo: z.string().optional(),
      memoMode: z.nativeEnum(MemoMode),
      clawbackEnabled: z.boolean().optional(),
      clawback: z
        .object({
          days: z.string(),
          hours: z.string(),
          minutes: z.string(),
          timestamp: z.number().optional(),
        })
        // Checked on the field rather than the whole form so the error updates
        // even while other fields are invalid. The value is cleared when
        // clawback is disabled, so it is only checked while enabled.
        .superRefine((value, ctx) => {
          const error = expirationError(value);
          if (error !== undefined) {
            ctx.addIssue({ code: z.ZodIssueCode.custom, message: error });
          }
        })
        .optional(),
    })
    .superRefine((values, ctx) => {
      if (
        values.memoMode === MemoMode.Hex &&
        values.memo &&
        !isValidHexBytes(values.memo)
      ) {
        ctx.addIssue({
          code: z.ZodIssueCode.custom,
          path: ['memo'],
          message: t`Memo must be valid hex bytes (even length, 0-9, a-f)`,
        });
      }
    });

  const form = useForm<z.infer<typeof formSchema>>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      memoMode: MemoMode.Text,
    },
  });
  const memoMode = form.watch('memoMode') || MemoMode.Text;

  const { handleScanOrPaste, handleScanImage } = useScannerOrClipboard(
    (scanResValue) => {
      form.setValue('address', scanResValue);
    },
  );

  const onSubmit = () => {
    const values = form.getValues();

    let memos: string[] = [];

    if (values.memo) {
      const memoBytes =
        values.memoMode === MemoMode.Hex
          ? fromHexStrict(values.memo)
          : utf8ToBytes(values.memo);

      memos = [toHex(memoBytes)];
    }

    setCurrentMemo(
      values.memo ? { mode: values.memoMode, value: values.memo } : undefined,
    );

    let clawback: number | null = null;

    if (values.clawbackEnabled && values.clawback) {
      try {
        clawback = futureExpirationTimestamp(values.clawback);
      } catch (error) {
        addError({ kind: 'invalid', reason: (error as Error).message });
        return;
      }
    }

    let result: Promise<TransactionResponse>;
    const address = values.address.trim();

    const amount = toMojos(values.amount.toString(), asset?.precision || 12);
    const fee = toMojos(
      values.fee?.toString() || '0',
      walletState.sync.unit.precision,
    );

    if (!assetId) {
      if (bulk) {
        result = withHandleConfirmation((confirmed_handles) =>
          commands.bulkSendXch({
            addresses: [...new Set(addressList(values.address))],
            amount,
            fee,
            memos,
            confirmed_handles,
          }),
        );
      } else {
        result = withHandleConfirmation((confirmed_handles) =>
          commands.sendXch({
            address,
            amount,
            fee,
            memos,
            clawback,
            confirmed_handles,
          }),
        );
      }
    } else {
      if (bulk) {
        result = withHandleConfirmation((confirmed_handles) =>
          commands.bulkSendCat({
            asset_id: assetId ?? '',
            addresses: [...new Set(addressList(values.address))],
            amount,
            fee,
            memos,
            confirmed_handles,
          }),
        );
      } else {
        result = withHandleConfirmation((confirmed_handles) =>
          commands.sendCat({
            asset_id: assetId ?? '',
            address,
            amount,
            fee,
            memos,
            clawback,
            confirmed_handles,
          }),
        );
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
                <Trans>Spendable Balance</Trans>
              </div>
              <div className='text-2xl font-medium mt-1'>
                <NumberFormat
                  value={fromMojos(asset.selectable_balance, asset.precision)}
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
                  <div className='flex items-center justify-between'>
                    <FormLabel>
                      <Trans>Address</Trans>
                    </FormLabel>
                    {!bulk && (
                      <WalletAddressPicker
                        onSelect={(address) =>
                          form.setValue('address', address)
                        }
                      />
                    )}
                  </div>
                  <FormControl>
                    {bulk ? (
                      <Textarea
                        autoCorrect='off'
                        autoCapitalize='off'
                        autoComplete='off'
                        placeholder={
                          xchandlesEnabled
                            ? t`Enter multiple distinct addresses or handles`
                            : t`Enter multiple distinct addresses`
                        }
                        {...field}
                      />
                    ) : (
                      <PasteInput
                        autoCorrect='off'
                        autoCapitalize='off'
                        autoComplete='off'
                        placeholder={
                          xchandlesEnabled
                            ? t`Enter address or @handle`
                            : t`Enter address`
                        }
                        onEndIconClick={handleScanOrPaste}
                        onScanImage={handleScanImage}
                        {...field}
                      />
                    )}
                  </FormControl>
                  {bulk ? (
                    <HandleResolutionList
                      values={addressList(field.value ?? '')}
                    />
                  ) : (
                    <HandleResolution value={field.value ?? ''} />
                  )}
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
                      <TokenAmountInput
                        {...field}
                        ticker={asset?.ticker}
                        precision={
                          asset?.precision ?? (assetId === null ? 12 : 3)
                        }
                        className='pr-12'
                        maxValue={
                          asset
                            ? BigNumber.max(
                                0,
                                fromMojos(
                                  asset.selectable_balance,
                                  asset.precision,
                                ).minus(
                                  assetId === null ? form.watch('fee') || 0 : 0,
                                ),
                              ).toString()
                            : undefined
                        }
                      />
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

              <div className='col-span-2'>
                <div className='flex items-center justify-between mb-2'>
                  <FormLabel>
                    <Trans>Memo (optional)</Trans>
                  </FormLabel>

                  <div className='flex items-center gap-2 text-sm'>
                    <Trans>Hex</Trans>

                    <Switch
                      checked={memoMode === MemoMode.Hex}
                      onCheckedChange={(checked) => {
                        form.setValue(
                          'memoMode',
                          checked ? MemoMode.Hex : MemoMode.Text,
                          {
                            shouldValidate: true,
                          },
                        );
                      }}
                    />
                  </div>
                </div>

                <FormField
                  control={form.control}
                  name='memo'
                  render={({ field }) => (
                    <FormItem>
                      <FormControl>
                        <Input
                          autoCorrect='off'
                          autoCapitalize='off'
                          autoComplete='off'
                          placeholder={
                            memoMode === MemoMode.Hex
                              ? t`e.g. ff00ab`
                              : t`Enter memo`
                          }
                          {...field}
                        />
                      </FormControl>
                      <FormMessage />
                    </FormItem>
                  )}
                />

                <div className='text-sm text-muted-foreground mt-2'>
                  {memoMode === MemoMode.Hex ? (
                    <Trans>Hex bytes</Trans>
                  ) : (
                    <Trans>UTF-8 string</Trans>
                  )}
                </div>
              </div>

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
                          form.clearErrors('clawback');
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
                        <ExpirationInput
                          value={form.watch('clawback') ?? emptyClawback}
                          onChange={(value) =>
                            form.setValue('clawback', value, {
                              shouldValidate: !!form.formState.errors.clawback,
                            })
                          }
                          formatLabel={(date) => t`Clawback until ${date}`}
                          error={form.formState.errors.clawback?.message}
                        />

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

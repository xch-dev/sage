import ConfirmationDialog from '@/components/ConfirmationDialog';
import Container from '@/components/Container';
import Header from '@/components/Header';
import { PasteInput } from '@/components/PasteInput';
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
import { Label } from '@/components/ui/label';
import { TokenAmountInput } from '@/components/ui/masked-input';
import { Switch } from '@/components/ui/switch';
import { Textarea } from '@/components/ui/textarea';
import { useErrors } from '@/hooks/useErrors';
import { useScannerOrClipboard } from '@/hooks/useScannerOrClipboard';
import { amount, positiveAmount } from '@/lib/formTypes';
import { fromMojos, toDecimal, toMojos } from '@/lib/utils';
import { useWalletState } from '@/state';
import { zodResolver } from '@hookform/resolvers/zod';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import BigNumber from 'bignumber.js';
import { useCallback, useEffect, useState } from 'react';
import { useForm } from 'react-hook-form';
import { useNavigate, useParams } from 'react-router-dom';
import * as z from 'zod';
import { CatRecord, commands, events, TransactionResponse } from '../bindings';
import { NumberFormat } from '@/components/NumberFormat';
import { toHex } from '@/lib/utils';
import { Input } from '@/components/ui/input';
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/components/ui/tooltip';
import { ArrowUpToLine } from 'lucide-react';
import { TokenConfirmation } from '@/components/confirmations/TokenConfirmation';

function stringToUint8Array(str: string): Uint8Array {
  return new TextEncoder().encode(str);
}

export default function Send() {
  const { asset_id: assetId } = useParams();
  const isXch = assetId === 'xch';
  const navigate = useNavigate();
  const walletState = useWalletState();
  const { addError } = useErrors();

  const [asset, setAsset] = useState<(CatRecord & { decimals: number }) | null>(
    null,
  );
  const [response, setResponse] = useState<TransactionResponse | null>(null);
  const [currentMemo, setCurrentMemo] = useState<string | undefined>(undefined);

  const [bulk, setBulk] = useState(false);

  const ticker = asset?.ticker || 'CAT';

  const updateCat = useCallback(
    () =>
      commands
        .getCat({ asset_id: assetId! })
        .then((data) => setAsset({ ...data.cat!, decimals: 3 }))
        .catch(addError),
    [assetId, addError],
  );

  useEffect(() => {
    if (isXch) {
      setAsset({
        asset_id: 'xch',
        name: 'Chia',
        description: 'The native token of the Chia blockchain.',
        ticker: walletState.sync.unit.ticker,
        decimals: walletState.sync.unit.decimals,
        balance: walletState.sync.balance,
        icon_url: 'https://icons.dexie.space/xch.webp',
        visible: true,
      });
    } else {
      updateCat();

      const unlisten = events.syncEvent.listen((event) => {
        const type = event.payload.type;
        if (
          type === 'coin_state' ||
          type === 'puzzle_batch_synced' ||
          type === 'cat_info'
        ) {
          updateCat();
        }
      });

      return () => {
        unlisten.then((u) => u());
      };
    }
  }, [
    updateCat,
    isXch,
    walletState.sync.balance,
    walletState.sync.unit.decimals,
    walletState.sync.unit.ticker,
  ]);

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
    amount: positiveAmount(asset?.decimals || 12).refine(
      (amount) =>
        asset
          ? BigNumber(amount).lte(toDecimal(asset.balance, asset.decimals))
          : true,
      'Amount exceeds balance',
    ),
    fee: amount(walletState.sync.unit.decimals).optional(),
    memo: z.string().optional(),
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

    let commandFn;
    if (isXch) {
      if (bulk) {
        commandFn = commands.bulkSendXch;
      } else {
        commandFn = commands.sendXch;
      }
    } else {
      if (bulk) {
        commandFn = commands.bulkSendCat;
      } else {
        commandFn = commands.sendCat;
      }
    }

    // Prepare common parameters
    const params: any = {
      amount: toMojos(values.amount.toString(), asset?.decimals || 12),
      fee: toMojos(
        values.fee?.toString() || '0',
        walletState.sync.unit.decimals,
      ),
      memos,
    };

    // Add asset_id for CAT tokens
    if (!isXch) {
      params.asset_id = assetId!;
    }

    // Handle address formatting for bulk operations
    if (bulk) {
      params.addresses = [...new Set(addressList(values.address))];
    } else {
      params.address = values.address;
    }

    // Execute the command
    commandFn(params)
      .then((confirmation) => setResponse(confirmation))
      .catch(addError);
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
                  value={fromMojos(asset.balance, asset.decimals)}
                  minimumFractionDigits={0}
                  maximumFractionDigits={asset.decimals}
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
                          className='pr-12 rounded-r-none z-10'
                        />
                        <TooltipProvider>
                          <Tooltip>
                            <TooltipTrigger asChild>
                              <Button
                                variant='outline'
                                size='icon'
                                type='button'
                                className='border-l-0 rounded-l-none flex-shrink-0'
                                onClick={() => {
                                  if (asset) {
                                    const maxAmount = fromMojos(
                                      asset.balance,
                                      asset.decimals,
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
                        <div className='pointer-events-none absolute inset-y-0 right-12 flex items-center pr-3'>
                          <span
                            className='text-gray-500 text-sm'
                            id='price-currency'
                          >
                            {asset?.ticker}
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
                      <Trans>Fee</Trans>
                    </FormLabel>
                    <FormControl>
                      <div className='relative'>
                        <TokenAmountInput {...field} className='pr-12' />
                        <div className='pointer-events-none absolute inset-y-0 right-0 flex items-center pr-3'>
                          <span
                            className='text-gray-500 text-sm'
                            id='price-currency'
                          >
                            {walletState.sync.unit.ticker}
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

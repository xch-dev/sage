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
import { useWalletState } from '@/state';
import { zodResolver } from '@hookform/resolvers/zod';
import { useCallback, useEffect, useState } from 'react';
import { useForm } from 'react-hook-form';
import { useNavigate, useParams } from 'react-router-dom';
import * as z from 'zod';
import {
  CatRecord,
  commands,
  events,
  SendXch,
  TransactionResponse,
} from '../bindings';
import Container from '../components/Container';
import { TokenAmountInput } from '@/components/ui/masked-input';

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

  const formSchema = z.object({
    address: z
      .string()
      .refine(
        (address) => commands.validateAddress(address).catch(addError),
        'Invalid address',
      ),
    amount: positiveAmount(asset?.decimals || 12),
    fee: amount(walletState.sync.unit.decimals).optional(),
  });

  const form = useForm<z.infer<typeof formSchema>>({
    resolver: zodResolver(formSchema),
  });

  const onSubmit = () => {
    const values = form.getValues();

    const command = isXch
      ? commands.sendXch
      : (req: SendXch) => {
          return commands.sendCat({ asset_id: assetId!, ...req });
        };

    command({
      address: values.address,
      amount: toMojos(values.amount.toString(), asset?.decimals || 12),
      fee: toMojos(
        values.fee?.toString() || '0',
        walletState.sync.unit.decimals,
      ),
    })
      .then((confirmation) => setResponse(confirmation))
      .catch(addError);
  };

  return (
    <>
      <Header
        title={`Send ${asset?.ticker || 'unknown asset'}`}
        back={() => navigate(-1)}
      />

      <Container className='max-w-xl'>
        <Form {...form}>
          <form onSubmit={form.handleSubmit(onSubmit)} className='space-y-4'>
            <FormField
              control={form.control}
              name='address'
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Address</FormLabel>
                  <FormControl>
                    <Input
                      autoCorrect='off'
                      autoCapitalize='off'
                      autoComplete='off'
                      placeholder='Enter address'
                      {...field}
                    />
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
                    <FormLabel>Amount</FormLabel>
                    <FormControl>
                      <div className='relative'>
                        <TokenAmountInput {...field} className='pr-12' />
                        <div className='pointer-events-none absolute inset-y-0 right-0 flex items-center pr-3'>
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
                    <FormLabel>Fee</FormLabel>
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
            </div>

            <Button type='submit'>Send {asset?.ticker}</Button>
          </form>
        </Form>
      </Container>

      <ConfirmationDialog
        response={response}
        close={() => setResponse(null)}
        onConfirm={() => navigate(-1)}
      />
    </>
  );
}

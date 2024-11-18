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
import { amount, positiveAmount } from '@/lib/formTypes';
import { useWalletState } from '@/state';
import { zodResolver } from '@hookform/resolvers/zod';
import { useCallback, useEffect, useState } from 'react';
import { useForm } from 'react-hook-form';
import { useNavigate, useParams } from 'react-router-dom';
import * as z from 'zod';
import {
  Amount,
  CatRecord,
  commands,
  Error,
  events,
  TransactionSummary,
} from '../bindings';
import Container from '../components/Container';
import ErrorDialog from '../components/ErrorDialog';

export default function Send() {
  const { asset_id: assetId } = useParams();
  const isXch = assetId === 'xch';

  const navigate = useNavigate();
  const walletState = useWalletState();

  const [asset, setAsset] = useState<(CatRecord & { decimals: number }) | null>(
    null,
  );
  const [error, setError] = useState<Error | null>(null);
  const [summary, setSummary] = useState<TransactionSummary | null>(null);

  const updateCat = useCallback(() => {
    commands.getCat(assetId!).then((result) => {
      if (result.status === 'ok') {
        setAsset({ ...result.data!, decimals: 3 });
      } else {
        console.error(result.error);
        setError(result.error);
      }
    });
  }, [assetId]);

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
        (address) =>
          commands
            .validateAddress(address)
            .then((result) => result.status === 'ok' && result.data),
        'Invalid address',
      ),
    amount: positiveAmount(asset?.decimals || 12),
    fee: amount(walletState.sync.unit.decimals).optional(),
  });

  const form = useForm<z.infer<typeof formSchema>>({
    resolver: zodResolver(formSchema),
  });

  const values = form.getValues();

  const onSubmit = () => {
    const command = isXch
      ? commands.send
      : (address: string, amount: Amount, fee: Amount) => {
          return commands.sendCat(assetId!, address, amount, fee);
        };

    command(
      values.address,
      values.amount.toString(),
      values.fee?.toString() || '0',
    ).then((confirmation) => {
      if (confirmation.status === 'error') {
        console.error(confirmation.error);
        return;
      }

      setSummary(confirmation.data);
    });
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
                        <Input
                          type='text'
                          placeholder='0.00'
                          {...field}
                          className='pr-12'
                        />

                        <div className='pointer-events-none absolute inset-y-0 right-0 flex items-center pr-3'>
                          <span
                            className='text-gray-500 sm:text-sm'
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
                        <Input
                          type='text'
                          placeholder='0.00'
                          {...field}
                          className='pr-12'
                        />

                        <div className='pointer-events-none absolute inset-y-0 right-0 flex items-center pr-3'>
                          <span
                            className='text-gray-500 sm:text-sm'
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

      <ErrorDialog error={error} setError={setError} />
      <ConfirmationDialog
        summary={summary}
        close={() => setSummary(null)}
        onConfirm={() => navigate(-1)}
        onError={(error) => {
          console.error(error);
          setError(error);
        }}
      />
    </>
  );
}

import ConfirmationDialog from '@/components/ConfirmationDialog';
import Header from '@/components/Header';
import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
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
  const [isConfirmOpen, setConfirmOpen] = useState(false);
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

  const onSubmit = () => {
    setConfirmOpen(true);
  };

  const values = form.getValues();

  const submit = () => {
    const command = isXch
      ? commands.send
      : (address: string, amount: Amount, fee: Amount, confirm: boolean) => {
          return commands.sendCat(assetId!, address, amount, fee, confirm);
        };

    command(
      values.address,
      values.amount.toString(),
      values.fee?.toString() || '0',
      true,
    ).then((confirmation) => {
      if (confirmation.status === 'error') {
        console.error(confirmation.error);
        return;
      } else {
        console.log(confirmation.data);
      }

      setSummary(confirmation.data);
    });
  };

  return (
    <>
      <Header title={`Send ${asset?.ticker}`} back={() => navigate(-1)} />

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

        <Dialog open={isConfirmOpen} onOpenChange={setConfirmOpen}>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>
                Are you sure you want to send {asset?.ticker}?
              </DialogTitle>
              <DialogDescription>
                This transaction cannot be reversed once it has been initiated.
              </DialogDescription>
            </DialogHeader>
            <div className='space-y-4'>
              <div>
                <h6 className='text-sm font-semibold'>Amount</h6>
                <p className='break-all'>
                  {values.amount} {asset?.ticker} (with a fee of{' '}
                  {values.fee || 0} {walletState.sync.unit.ticker})
                </p>
              </div>
              <div>
                <h6 className='text-sm font-semibold'>Address</h6>
                <p className='break-all'>{values.address}</p>
              </div>
            </div>
            <DialogFooter>
              <Button variant='outline' onClick={() => setConfirmOpen(false)}>
                Cancel
              </Button>
              <Button
                onClick={() => {
                  setConfirmOpen(false);
                  submit();
                }}
              >
                Confirm
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      </Container>

      <ErrorDialog error={error} setError={setError} />
      <ConfirmationDialog
        summary={summary}
        close={() => setSummary(null)}
        confirm={async () => {
          const command = isXch
            ? commands.send
            : (
                address: string,
                amount: Amount,
                fee: Amount,
                confirm: boolean,
              ) => {
                return commands.sendCat(
                  assetId!,
                  address,
                  amount,
                  fee,
                  confirm,
                );
              };

          const result = await command(
            values.address,
            values.amount.toString(),
            values.fee?.toString() || '0',
            false,
          );

          if (result.status === 'ok') {
            navigate(-1);
          } else {
            console.error(result.error);
            setError(result.error);
          }
        }}
      />
    </>
  );
}

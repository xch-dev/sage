import { useScannerOrClipboard } from '@/hooks/useScannerOrClipboard';
import { amount } from '@/lib/formTypes';
import { useWalletState } from '@/state';
import { zodResolver } from '@hookform/resolvers/zod';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import BigNumber from 'bignumber.js';
import { PropsWithChildren } from 'react';
import { useForm } from 'react-hook-form';
import { z } from 'zod';
import { PasteInput } from './PasteInput';
import { WalletAddressPicker } from './WalletAddressPicker';
import { Button } from './ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from './ui/dialog';
import {
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from './ui/form';
import { FeeAmountInput } from './ui/masked-input';

export interface TransferDialogProps {
  title: string;
  open: boolean;
  setOpen: (open: boolean) => void;
  onSubmit: (address: string, fee: string) => void;
}

export function TransferDialog({
  title,
  open,
  setOpen,
  onSubmit,
  children,
}: PropsWithChildren<TransferDialogProps>) {
  const walletState = useWalletState();

  const schema = z.object({
    address: z.string().min(1, t`Address is required`),
    fee: amount(walletState.sync.unit.precision).refine(
      (amount) =>
        BigNumber(walletState.sync.selectable_balance).gte(amount || 0),
      t`Not enough funds to cover the fee`,
    ),
  });

  const form = useForm<z.infer<typeof schema>>({
    resolver: zodResolver(schema),
    defaultValues: {
      address: '',
    },
  });

  const { handleScanOrPaste } = useScannerOrClipboard((scanResValue) => {
    form.setValue('address', scanResValue);
  });

  const handler = (values: z.infer<typeof schema>) => {
    onSubmit(values.address, values.fee);
  };

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{title}</DialogTitle>
          <DialogDescription>{children}</DialogDescription>
        </DialogHeader>
        <Form {...form}>
          <form onSubmit={form.handleSubmit(handler)} className='space-y-4'>
            <FormField
              control={form.control}
              name='address'
              render={({ field }) => (
                <FormItem>
                  <div className='flex items-center justify-between'>
                    <FormLabel>
                      <Trans>Address</Trans>
                    </FormLabel>
                    <WalletAddressPicker
                      onSelect={(address) => form.setValue('address', address)}
                    />
                  </div>
                  <FormControl>
                    <PasteInput
                      {...field}
                      placeholder={t`Enter address`}
                      onEndIconClick={handleScanOrPaste}
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
                    <FeeAmountInput {...field} />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />
            <DialogFooter className='gap-2'>
              <Button
                type='button'
                variant='outline'
                onClick={() => setOpen(false)}
              >
                <Trans>Cancel</Trans>
              </Button>
              <Button type='submit'>
                <Trans>Transfer</Trans>
              </Button>
            </DialogFooter>
          </form>
        </Form>
      </DialogContent>
    </Dialog>
  );
}

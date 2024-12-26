import { amount } from '@/lib/formTypes';
import { useWalletState } from '@/state';
import { zodResolver } from '@hookform/resolvers/zod';
import BigNumber from 'bignumber.js';
import { PropsWithChildren } from 'react';
import { useForm } from 'react-hook-form';
import { z } from 'zod';
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
import { Input } from './ui/input';
import { TokenAmountInput } from './ui/masked-input';
import { AddressInput } from './AddressInput';
import { isValidXchName, resolveXchName } from '@/utils/namesdao';

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
    address: z.string().min(1, 'Address is required').refine(
      async (address) => {
        if (isValidXchName(address)) {
          const resolved = await resolveXchName(address);
          return !!resolved;
        }
        return true;
      },
      'Invalid .xch name',
    ),
    fee: amount(walletState.sync.unit.decimals).refine(
      (amount) => BigNumber(walletState.sync.balance).gte(amount || 0),
      'Not enough funds to cover the fee',
    ),
  });

  const form = useForm<z.infer<typeof schema>>({
    resolver: zodResolver(schema),
  });

  const handleSubmit = async (values: z.infer<typeof schema>) => {
    let targetAddress = values.address;

    // Resolve .xch name if needed
    if (isValidXchName(targetAddress)) {
      const resolved = await resolveXchName(targetAddress);
      if (!resolved) {
        form.setError('address', { message: 'Failed to resolve .xch name' });
        return;
      }
      targetAddress = resolved;
    }

    onSubmit(targetAddress, values.fee?.toString() || '0');
  };

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{title}</DialogTitle>
          <DialogDescription>{children}</DialogDescription>
        </DialogHeader>
        <Form {...form}>
          <form onSubmit={form.handleSubmit(handleSubmit)} className='space-y-4'>
            <FormField
              control={form.control}
              name='address'
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Address</FormLabel>
                  <FormControl>
                    <AddressInput
                      autoCorrect='off'
                      autoCapitalize='off'
                      autoComplete='off'
                      placeholder='Enter address or .xch name'
                      {...field}
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
                  <FormLabel>Network Fee</FormLabel>
                  <FormControl>
                    <TokenAmountInput {...field} />
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
                Cancel
              </Button>
              <Button type='submit'>Transfer</Button>
            </DialogFooter>
          </form>
        </Form>
      </DialogContent>
    </Dialog>
  );
}

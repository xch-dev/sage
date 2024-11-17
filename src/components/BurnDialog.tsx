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

export interface BurnDialogProps {
  title: string;
  open: boolean;
  setOpen: (open: boolean) => void;
  onSubmit: (fee: string) => void;
}

export function BurnDialog({
  title,
  open,
  setOpen,
  onSubmit,
  children,
}: PropsWithChildren<BurnDialogProps>) {
  const walletState = useWalletState();

  const schema = z.object({
    fee: amount(walletState.sync.unit.decimals).refine(
      (amount) => BigNumber(walletState.sync.balance).gte(amount || 0),
      'Not enough funds to cover the fee',
    ),
  });

  const form = useForm<z.infer<typeof schema>>({
    resolver: zodResolver(schema),
    defaultValues: {
      fee: '0',
    },
  });

  const handler = (values: z.infer<typeof schema>) => {
    onSubmit(values.fee);
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
              name='fee'
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Network Fee</FormLabel>
                  <FormControl>
                    <Input {...field} />
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

import { useDids } from '@/hooks/useDids';
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
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from './ui/select';

export interface AssignNftDialogProps {
  title: string;
  open: boolean;
  setOpen: (open: boolean) => void;
  onSubmit: (profile: string, fee: string) => void;
}

export function AssignNftDialog({
  title,
  open,
  setOpen,
  onSubmit,
  children,
}: PropsWithChildren<AssignNftDialogProps>) {
  const walletState = useWalletState();

  const { dids } = useDids();

  const schema = z.object({
    profile: z.string().min(1, 'Profile is required'),
    fee: amount(walletState.sync.unit.decimals).refine(
      (amount) => BigNumber(walletState.sync.balance).gte(amount || 0),
      'Not enough funds to cover the fee',
    ),
  });

  const form = useForm<z.infer<typeof schema>>({
    resolver: zodResolver(schema),
    defaultValues: {
      profile: '',
      fee: '0',
    },
  });

  const handler = (values: z.infer<typeof schema>) => {
    onSubmit(values.profile, values.fee);
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
              name='profile'
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Profile</FormLabel>
                  <FormControl>
                    <Select value={field.value} onValueChange={field.onChange}>
                      <SelectTrigger
                        id='profile'
                        aria-label='Select profile'
                        className='truncate max-w-full'
                      >
                        <SelectValue placeholder='Select profile' />
                      </SelectTrigger>
                      <SelectContent>
                        {dids
                          .filter((did) => did.visible)
                          .map((did) => {
                            return (
                              <SelectItem
                                key={did.launcher_id}
                                value={did.launcher_id}
                              >
                                {did.name ?? did.launcher_id}
                              </SelectItem>
                            );
                          })}
                      </SelectContent>
                    </Select>
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

import { useDids } from '@/hooks/useDids';
import { amount } from '@/lib/formTypes';
import { useWalletState } from '@/state';
import { zodResolver } from '@hookform/resolvers/zod';
import BigNumber from 'bignumber.js';
import { PropsWithChildren } from 'react';
import { useForm } from 'react-hook-form';
import { z } from 'zod';
import { Trans } from '@lingui/react/macro';
import { t } from '@lingui/core/macro';
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
import { TokenAmountInput } from './ui/masked-input';

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
    profile: z.string().min(1, t`Profile is required`),
    fee: amount(walletState.sync.unit.decimals).refine(
      (amount) => BigNumber(walletState.sync.balance).gte(amount || 0),
      t`Not enough funds to cover the fee`,
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
          <form
            onSubmit={form.handleSubmit(handler)}
            className='space-y-4 overflow-hidden p-[1px]'
          >
            <FormField
              control={form.control}
              name='profile'
              render={({ field }) => (
                <FormItem>
                  <FormLabel>
                    <Trans>Profile</Trans>
                  </FormLabel>
                  <FormControl>
                    <Select value={field.value} onValueChange={field.onChange}>
                      <SelectTrigger
                        id='profile'
                        aria-label={t`Select profile`}
                      >
                        <SelectValue placeholder={t`Select profile`} />
                      </SelectTrigger>
                      <SelectContent className='max-w-[var(--radix-select-trigger-width)]'>
                        {dids
                          .filter((did) => did.visible)
                          .map((did) => {
                            return (
                              <SelectItem
                                key={did.launcher_id}
                                value={did.launcher_id}
                              >
                                {did.name ??
                                  `${did.launcher_id.slice(0, 14)}...${did.launcher_id.slice(-4)}`}
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
                  <FormLabel>
                    <Trans>Network Fee</Trans>
                  </FormLabel>
                  <FormControl>
                    <TokenAmountInput
                      {...field}
                      aria-label={t`Network fee amount`}
                    />
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

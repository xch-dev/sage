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
import { FeeAmountInput } from '@/components/ui/masked-input';
import { Trans } from '@lingui/react/macro';
import { UseFormReturn } from 'react-hook-form';

interface CancelOfferDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  form: UseFormReturn<any>;
  onSubmit: (values: any) => void;
  title?: React.ReactNode;
  description?: React.ReactNode;
  feeLabel?: React.ReactNode;
}

export function CancelOfferDialog({
  open,
  onOpenChange,
  form,
  onSubmit,
  title,
  description,
  feeLabel,
}: CancelOfferDialogProps) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{title || <Trans>Cancel offer?</Trans>}</DialogTitle>
          <DialogDescription>
            {description || (
              <Trans>
                This will cancel the offer on-chain with a transaction,
                preventing it from being taken even if someone has the original
                offer file.
              </Trans>
            )}
          </DialogDescription>
        </DialogHeader>
        <Form {...form}>
          <form onSubmit={form.handleSubmit(onSubmit)} className='space-y-4'>
            <FormField
              control={form.control}
              name='fee'
              render={({ field }) => (
                <FormItem>
                  <FormLabel>
                    {feeLabel || <Trans>Network Fee</Trans>}
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
                onClick={() => onOpenChange(false)}
              >
                <Trans>Cancel</Trans>
              </Button>
              <Button type='submit'>
                <Trans>Submit</Trans>
              </Button>
            </DialogFooter>
          </form>
        </Form>
      </DialogContent>
    </Dialog>
  );
}

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
import { LoadingButton } from '@/components/ui/loading-button';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { UseFormReturn } from 'react-hook-form';

interface CancelOfferDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  form: UseFormReturn<{ fee: string }>;
  onSubmit: (values: { fee: string }) => void;
  title?: React.ReactNode;
  description?: React.ReactNode;
  feeLabel?: React.ReactNode;
  pending?: boolean;
}

export function CancelOfferDialog({
  open,
  onOpenChange,
  form,
  onSubmit,
  title,
  description,
  feeLabel,
  pending = false,
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
                disabled={pending}
                onClick={() => onOpenChange(false)}
              >
                <Trans>Cancel</Trans>
              </Button>
              <LoadingButton
                type='submit'
                loading={pending}
                loadingText={t`Submitting`}
              >
                <Trans>Submit</Trans>
              </LoadingButton>
            </DialogFooter>
          </form>
        </Form>
      </DialogContent>
    </Dialog>
  );
}

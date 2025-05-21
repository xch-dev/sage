import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { Trans } from '@lingui/react/macro';
import { LoaderCircleIcon } from 'lucide-react';

interface OfferCreationProgressDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  createdOffers: string[];
  onOk: () => void;
  isProcessing?: boolean;
  onCancel?: () => void;
  isMultipleOffers?: boolean;
  isUploading?: boolean;
}

export function OfferCreationProgressDialog({
  open,
  onOpenChange,
  createdOffers,
  onOk,
  isProcessing = false,
  onCancel,
  isMultipleOffers = false,
  isUploading = false,
}: OfferCreationProgressDialogProps) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>
            {isProcessing ? (
              <div className='flex items-center gap-2'>
                <LoaderCircleIcon
                  className='h-4 w-4 animate-spin'
                  aria-hidden='true'
                />
                {isMultipleOffers ? (
                  <Trans>Creating Offers</Trans>
                ) : (
                  <Trans>Creating Offer</Trans>
                )}
              </div>
            ) : createdOffers.length > 1 ? (
              <Trans>Offers Created</Trans>
            ) : (
              <Trans>Offer Created</Trans>
            )}
          </DialogTitle>
          <DialogDescription>
            {isProcessing ? (
              <Trans>
                Please wait while{' '}
                {isMultipleOffers ? 'your offers are' : 'your offer is'} being
                created{isUploading ? ' and uploaded' : ''}...
              </Trans>
            ) : createdOffers.length > 1 ? (
              <Trans>
                {createdOffers.length} offers have been created and imported
                successfully. You will now be redirected to the offers page
                where you can view the details of each offer.
              </Trans>
            ) : (
              <Trans>
                Your offer has been created and imported successfully. You will
                now be redirected to the offers page where you can view its
                details.
              </Trans>
            )}
          </DialogDescription>
        </DialogHeader>
        <DialogFooter>
          {isProcessing ? (
            <Button variant='outline' onClick={onCancel}>
              <Trans>Cancel</Trans>
            </Button>
          ) : (
            <Button onClick={onOk}>
              <Trans>Ok</Trans>
            </Button>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

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

interface OfferCreationProgressDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  createdOffers: string[];
  onOk: () => void;
}

export function OfferCreationProgressDialog({
  open,
  onOpenChange,
  createdOffers,
  onOk,
}: OfferCreationProgressDialogProps) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>
            {createdOffers.length > 1 ? (
              <Trans>Offers Created</Trans>
            ) : (
              <Trans>Offer Created</Trans>
            )}
          </DialogTitle>
          <DialogDescription>
            {createdOffers.length > 1 ? (
              <Trans>
                {createdOffers.length} offers have been created and imported
                successfully. You will now be redirected to the offers page
                where you can view the details of each offer.
              </Trans>
            ) : (
              <Trans>
                The offer has been created and imported successfully. You will
                now be redirected to the offers page where you can view its
                details.
              </Trans>
            )}
          </DialogDescription>
        </DialogHeader>
        <DialogFooter>
          <Button onClick={onOk}>
            <Trans>Ok</Trans>
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

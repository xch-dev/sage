import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Trans } from '@lingui/react/macro';

interface DeleteOfferDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onDelete: () => void;
  offerCount: number;
}

export function DeleteOfferDialog({
  open,
  onOpenChange,
  onDelete,
  offerCount,
}: DeleteOfferDialogProps) {
  const isMultiple = offerCount > 1;

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>
            {isMultiple ? (
              <Trans>Delete {offerCount} offer records?</Trans>
            ) : (
              <Trans>Delete offer record?</Trans>
            )}
          </DialogTitle>
          <DialogDescription>
            {isMultiple ? (
              <Trans>
                This will delete {offerCount} offers from the wallet, but if
                they're shared externally they can still be accepted. The only
                way to truly cancel public offers is by spending one or more of
                their coins.
              </Trans>
            ) : (
              <Trans>
                This will delete the offer from the wallet, but if it's shared
                externally it can still be accepted. The only way to truly
                cancel a public offer is by spending one or more of its coins.
              </Trans>
            )}
          </DialogDescription>
        </DialogHeader>
        <DialogFooter>
          <Button variant='outline' onClick={() => onOpenChange(false)}>
            <Trans>Cancel</Trans>
          </Button>
          <Button variant='destructive' onClick={onDelete}>
            {isMultiple ? <Trans>Delete All</Trans> : <Trans>Delete</Trans>}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

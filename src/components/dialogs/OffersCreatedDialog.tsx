import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Label } from '@/components/ui/label';
import { Button } from '@/components/ui/button';
import { CopyBox } from '@/components/CopyBox';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';

interface OffersCreatedDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  createdOffer: string;
  createdOffers: string[];
  selectedDialogOffer: string;
  setSelectedDialogOffer: (offer: string) => void;
  mintGardenLink: string;
  canUploadToMintGarden: boolean;
  network: string | null;
  onMintGardenUpload: () => void;
  onOk: () => void;
}

export function OffersCreatedDialog({
  open,
  onOpenChange,
  createdOffer,
  createdOffers,
  selectedDialogOffer,
  setSelectedDialogOffer,
  mintGardenLink,
  canUploadToMintGarden,
  network,
  onMintGardenUpload,
  onOk,
}: OffersCreatedDialogProps) {
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

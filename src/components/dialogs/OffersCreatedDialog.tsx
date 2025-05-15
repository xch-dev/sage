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
  dexieLink: string;
  mintGardenLink: string;
  canUploadToMintGarden: boolean;
  network: string | null;
  onDexieUpload: () => void;
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
  dexieLink,
  mintGardenLink,
  canUploadToMintGarden,
  network,
  onDexieUpload,
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
                successfully. Select an offer to view its details or copy it.
              </Trans>
            ) : (
              <Trans>
                The offer has been created and imported successfully. You can
                copy the offer file below and send it to the intended recipient
                or make it public to be accepted by anyone.
              </Trans>
            )}
            {(createdOffers.length > 1 ||
              (createdOffers.length === 0 && createdOffer)) && (
              <div className='mt-2'>
                <Label>
                  {createdOffers.length > 1 ? (
                    <Trans>Select Offer</Trans>
                  ) : (
                    <Trans>Offer File</Trans>
                  )}
                </Label>
                {createdOffers.length > 1 ? (
                  <select
                    className='w-full mt-1 p-2 border rounded'
                    value={selectedDialogOffer}
                    onChange={(e) => setSelectedDialogOffer(e.target.value)}
                  >
                    {createdOffers.map((o, i) => (
                      <option key={i} value={o}>
                        {t`Offer ${i + 1}`}
                      </option>
                    ))}
                  </select>
                ) : null}
                <CopyBox
                  title={
                    createdOffers.length > 1
                      ? t`Offer ${createdOffers.indexOf(selectedDialogOffer) + 1}`
                      : t`Offer File`
                  }
                  value={selectedDialogOffer}
                  className='mt-2'
                />
              </div>
            )}

            {network !== 'unknown' &&
              (createdOffers.length > 0 || createdOffer) && (
                <div className='flex flex-col gap-2 mt-2'>
                  <div className='grid grid-cols-2 gap-2'>
                    <Button
                      variant='outline'
                      className='text-neutral-800 dark:text-neutral-200'
                      onClick={onDexieUpload}
                    >
                      <img
                        src='https://raw.githubusercontent.com/dexie-space/dexie-kit/refs/heads/main/svg/duck.svg'
                        className='h-4 w-4 mr-2'
                        alt='Dexie logo'
                      />
                      {dexieLink ? t`Dexie Link` : t`Upload to Dexie`}
                    </Button>

                    {canUploadToMintGarden && (
                      <Button
                        variant='outline'
                        className='text-neutral-800 dark:text-neutral-200'
                        onClick={onMintGardenUpload}
                      >
                        <img
                          src='https://mintgarden.io/mint-logo.svg'
                          className='h-4 w-4 mr-2'
                          alt='MintGarden logo'
                        />
                        {mintGardenLink
                          ? t`MintGarden Link`
                          : t`Upload to MintGarden`}
                      </Button>
                    )}
                  </div>
                </div>
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

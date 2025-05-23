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
import { useEffect, useState } from 'react';
import { commands, NetworkKind } from '@/bindings';
import { uploadToDexie, uploadToMintGarden } from '@/lib/offerUpload';
import { useErrors } from '@/hooks/useErrors';
import { useOfferProcessor } from '@/hooks/useOfferProcessor';
import { OfferState } from '@/state';
import { useNavigate } from 'react-router-dom';
import { CustomError } from '@/contexts/ErrorContext';

const delay = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

interface OfferCreationProgressDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  offerState: OfferState;
  splitNftOffers: boolean;
  autoUploadToDexie: boolean;
  autoUploadToMintGarden: boolean;
  clearOfferState: () => void;
}

export function OfferCreationProgressDialog({
  open,
  onOpenChange,
  offerState,
  splitNftOffers,
  autoUploadToDexie,
  autoUploadToMintGarden,
  clearOfferState,
}: OfferCreationProgressDialogProps) {
  const { addError } = useErrors();
  const navigate = useNavigate();
  const [network, setNetwork] = useState<NetworkKind | null>(null);
  const [isUploadingToDexie, setIsUploadingToDexie] = useState(false);
  const [isUploadingToMintGarden, setIsUploadingToMintGarden] = useState(false);
  const [hasStartedProcessing, setHasStartedProcessing] = useState(false);
  const [isCanceling, setIsCanceling] = useState(false);
  const [currentStep, setCurrentStep] = useState<
    'creating' | 'dexie' | 'mintgarden'
  >('creating');
  const [currentIndex, setCurrentIndex] = useState(0);
  const totalOffers = splitNftOffers
    ? offerState.offered.nfts.filter((n) => n).length
    : 1;

  const {
    createdOffers,
    isProcessing,
    processOffer,
    clearProcessedOffers,
    cancelProcessing,
  } = useOfferProcessor({
    offerState,
    splitNftOffers,
    onProcessingEnd: () => {
      // Don't auto-close on success
    },
    onProgress: (index: number) => {
      setCurrentIndex(index);
    },
  });

  useEffect(() => {
    commands.getNetwork({}).then((data) => setNetwork(data.kind));
  }, []);

  // Handle uploads when offers are created
  useEffect(() => {
    if (createdOffers.length > 0 && network) {
      let isMounted = true;

      const uploadToDexieWithDelay = async () => {
        if (!autoUploadToDexie) return;
        setIsUploadingToDexie(true);
        setCurrentStep('dexie');
        for (const [index, individualOffer] of createdOffers.entries()) {
          if (!isMounted) break;
          setCurrentIndex(index);
          try {
            await uploadToDexie(individualOffer, network === 'testnet');
            if (index < createdOffers.length - 1) {
              await delay(500);
            }
          } catch (error) {
            if (isMounted) {
              addError({
                kind: 'upload',
                reason: `Failed to auto-upload offer ${index + 1} to Dexie: ${error}`,
              });
            }
          }
        }
        if (isMounted) {
          setIsUploadingToDexie(false);
        }
      };

      const uploadToMintGardenWithDelay = async () => {
        if (!autoUploadToMintGarden) return;
        setIsUploadingToMintGarden(true);
        setCurrentStep('mintgarden');
        for (const [index, individualOffer] of createdOffers.entries()) {
          if (!isMounted) break;
          setCurrentIndex(index);
          try {
            await uploadToMintGarden(individualOffer, network === 'testnet');
            if (index < createdOffers.length - 1) {
              await delay(500);
            }
          } catch (error) {
            if (isMounted) {
              addError({
                kind: 'upload',
                reason: `Failed to auto-upload offer ${index + 1} to MintGarden: ${error}`,
              });
            }
          }
        }
        if (isMounted) {
          setIsUploadingToMintGarden(false);
        }
      };

      const handleUploads = async () => {
        if (autoUploadToDexie) {
          await uploadToDexieWithDelay();
        }
        if (autoUploadToMintGarden) {
          await uploadToMintGardenWithDelay();
        }
      };

      handleUploads();

      return () => {
        isMounted = false;
      };
    }
  }, [
    createdOffers,
    network,
    addError,
    autoUploadToDexie,
    autoUploadToMintGarden,
  ]);

  // Start processing when dialog opens
  useEffect(() => {
    if (open && !hasStartedProcessing && !isCanceling) {
      setHasStartedProcessing(true);
      setCurrentStep('creating');
      const startProcessing = async () => {
        try {
          await processOffer();
        } catch (error) {
          if (
            error &&
            typeof error === 'object' &&
            'kind' in error &&
            'reason' in error
          ) {
            addError(error as CustomError);
          } else {
            addError({
              kind: 'invalid',
              reason: error instanceof Error ? error.message : 'Unknown error',
            });
          }
          onOpenChange(false);
        }
      };
      startProcessing();
    }
  }, [
    open,
    hasStartedProcessing,
    isCanceling,
    processOffer,
    addError,
    onOpenChange,
  ]);

  // Reset processing state when dialog closes
  useEffect(() => {
    if (!open) {
      setHasStartedProcessing(false);
      setIsCanceling(false);
      setCurrentStep('creating');
      setCurrentIndex(0);
    }
  }, [open]);

  const handleClose = (isOpen: boolean) => {
    if (!isOpen) {
      // Just close the dialog, don't clear state or navigate
      onOpenChange(false);
    }
  };

  const handleCancel = async () => {
    setIsCanceling(true);
    await cancelProcessing();
    clearProcessedOffers();
    onOpenChange(false);
  };

  const handleDone = () => {
    clearProcessedOffers();
    clearOfferState();
    navigate('/offers', { replace: true });
  };

  const getProgressMessage = () => {
    if (isProcessing) {
      switch (currentStep) {
        case 'creating':
          return (
            <Trans>
              Creating offer {currentIndex + 1} of {totalOffers}...
            </Trans>
          );
        case 'dexie':
          return (
            <Trans>
              Uploading offer {currentIndex + 1} of {totalOffers} to Dexie...
            </Trans>
          );
        case 'mintgarden':
          return (
            <Trans>
              Uploading offer {currentIndex + 1} of {totalOffers} to
              MintGarden...
            </Trans>
          );
      }
    }
    return null;
  };

  return (
    <Dialog open={open} onOpenChange={handleClose}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>
            {isProcessing ? (
              <div className='flex items-center gap-2'>
                <LoaderCircleIcon
                  className='h-4 w-4 animate-spin'
                  aria-hidden='true'
                />
                {splitNftOffers ? (
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
              <div className='space-y-2'>
                <p>
                  <Trans>
                    Please wait while{' '}
                    {splitNftOffers ? 'your offers are' : 'your offer is'} being
                    created
                    {autoUploadToDexie || autoUploadToMintGarden
                      ? ' and uploaded'
                      : ''}
                    ...
                  </Trans>
                </p>
                <p className='text-sm text-muted-foreground'>
                  {getProgressMessage()}
                </p>
              </div>
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
            <Button variant='outline' onClick={handleCancel}>
              <Trans>Cancel</Trans>
            </Button>
          ) : (
            <Button onClick={handleDone}>
              <Trans>Done</Trans>
            </Button>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

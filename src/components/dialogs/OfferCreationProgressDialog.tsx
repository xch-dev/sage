import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { CustomError } from '@/contexts/ErrorContext';
import { useErrors } from '@/hooks/useErrors';
import { useNetwork } from '@/hooks/useNetwork';
import { useOfferProcessor } from '@/hooks/useOfferProcessor';
import { marketplaces } from '@/lib/marketplaces';
import { OfferState } from '@/state';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { LoaderCircleIcon } from 'lucide-react';
import { useEffect, useState } from 'react';

const delay = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

interface OfferCreationProgressDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  offerState: OfferState;
  splitNftOffers: boolean;
  enabledMarketplaces?: Record<string, boolean>;
  clearOfferState: (offers: string[]) => void;
  isSwap?: boolean;
}

export function OfferCreationProgressDialog({
  open,
  onOpenChange,
  offerState,
  splitNftOffers,
  enabledMarketplaces,
  clearOfferState,
  isSwap,
}: OfferCreationProgressDialogProps) {
  const { addError } = useErrors();
  const { isTestnet, isUnknown } = useNetwork();
  const [isUploading, setIsUploading] = useState(false);
  const [hasStartedProcessing, setHasStartedProcessing] = useState(false);
  const [isCanceling, setIsCanceling] = useState(false);
  const [currentStep, setCurrentStep] = useState<'creating' | 'uploading'>(
    'creating',
  );
  const [currentMarketplaceIndex, setCurrentMarketplaceIndex] = useState(0);
  const [currentOfferIndex, setCurrentOfferIndex] = useState(0);
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
      setCurrentOfferIndex(index);
    },
  });

  // Handle uploads when offers are created
  useEffect(() => {
    if (
      createdOffers.length > 0 &&
      !isUnknown &&
      !isProcessing &&
      !isCanceling
    ) {
      let isMounted = true;

      const uploadToMarketplaces = async () => {
        const enabledMarketplaceConfigs = marketplaces.filter(
          (marketplace) => enabledMarketplaces?.[marketplace.id],
        );

        if (enabledMarketplaceConfigs.length === 0) {
          return;
        }

        setIsUploading(true);
        setCurrentStep('uploading');

        for (const [
          marketplaceIndex,
          marketplace,
        ] of enabledMarketplaceConfigs.entries()) {
          if (!isMounted || isCanceling) break;
          setCurrentMarketplaceIndex(marketplaceIndex);

          for (const [offerIndex, individualOffer] of createdOffers.entries()) {
            if (!isMounted || isCanceling) break;
            setCurrentOfferIndex(offerIndex);
            try {
              await marketplace.uploadToMarketplace(individualOffer, isTestnet);
              if (offerIndex < createdOffers.length - 1) {
                // rate limit
                await delay(500);
              }
            } catch (error) {
              if (isMounted) {
                const offerNumber = offerIndex + 1;
                const marketplaceName = marketplace.name;
                const message = error as string;
                addError({
                  kind: 'upload',
                  reason: t`Failed to auto-upload offer ${offerNumber} to ${marketplaceName}. Stopping.: ${message}`,
                });
                // typically if one fails the rest will fail too
                break;
              }
            }
          }
        }

        if (isMounted && !isCanceling) {
          setIsUploading(false);
        }
      };

      uploadToMarketplaces();

      return () => {
        isMounted = false;
      };
    }
  }, [
    createdOffers,
    isTestnet,
    isUnknown,
    addError,
    enabledMarketplaces,
    isProcessing,
    isCanceling,
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
              reason: error instanceof Error ? error.message : t`Unknown error`,
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
      setCurrentOfferIndex(0);
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
    if (isProcessing) {
      cancelProcessing();
    }
    clearProcessedOffers();
    setIsUploading(false);
    onOpenChange(false);
  };

  const handleDone = () => {
    clearProcessedOffers();
    clearOfferState(createdOffers);
  };

  const createdOfferCount = createdOffers.length;

  const uploadedToMarketplaces = Object.values(enabledMarketplaces ?? {}).some(
    Boolean,
  );

  // Each combination is spelled out as a whole sentence so translators get
  // complete phrases rather than English fragments stitched together.
  const getWaitMessage = () => {
    if (currentStep === 'creating') {
      if (uploadedToMarketplaces) {
        return splitNftOffers ? (
          <Trans>
            Please wait while your offers are being created and uploaded...
          </Trans>
        ) : (
          <Trans>
            Please wait while your offer is being created and uploaded...
          </Trans>
        );
      }
      return splitNftOffers ? (
        <Trans>Please wait while your offers are being created...</Trans>
      ) : (
        <Trans>Please wait while your offer is being created...</Trans>
      );
    }
    return splitNftOffers ? (
      <Trans>Please wait while your offers are being uploaded...</Trans>
    ) : (
      <Trans>Please wait while your offer is being uploaded...</Trans>
    );
  };

  const getProgressMessage = () => {
    if (isProcessing || isUploading) {
      const offerNumber = currentOfferIndex + 1;
      if (currentStep === 'creating') {
        return (
          <Trans>
            Creating offer {offerNumber} of {totalOffers}...
          </Trans>
        );
      } else if (currentStep === 'uploading') {
        const enabledMarketplaceConfigs = marketplaces.filter(
          (marketplace) => enabledMarketplaces?.[marketplace.id],
        );
        const currentMarketplace =
          enabledMarketplaceConfigs[currentMarketplaceIndex];
        const marketplaceName = currentMarketplace.name;
        return (
          <Trans>
            Uploading offer {offerNumber} of {totalOffers} to {marketplaceName}
            ...
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
            {isProcessing || isUploading ? (
              <div className='flex items-center gap-2'>
                <LoaderCircleIcon
                  className='h-4 w-4 animate-spin'
                  aria-hidden='true'
                />
                {currentStep === 'creating' ? (
                  splitNftOffers ? (
                    <Trans>Creating Offers</Trans>
                  ) : (
                    <Trans>Creating Offer</Trans>
                  )
                ) : splitNftOffers ? (
                  <Trans>Uploading Offers</Trans>
                ) : (
                  <Trans>Uploading Offer</Trans>
                )}
              </div>
            ) : createdOffers.length > 1 ? (
              <Trans>Offers Created</Trans>
            ) : (
              <Trans>Offer Created</Trans>
            )}
          </DialogTitle>
          <DialogDescription>
            {isProcessing || isUploading ? (
              <div className='space-y-2'>
                <p>{getWaitMessage()}</p>
                <p className='text-sm text-muted-foreground'>
                  {getProgressMessage()}
                </p>
              </div>
            ) : createdOffers.length > 1 ? (
              uploadedToMarketplaces ? (
                <Trans>
                  {createdOfferCount} offers have been created and imported
                  successfully and uploaded to the selected marketplaces. You
                  will now be redirected to the offers page where you can view
                  the details of each offer.
                </Trans>
              ) : (
                <Trans>
                  {createdOfferCount} offers have been created and imported
                  successfully. You will now be redirected to the offers page
                  where you can view the details of each offer.
                </Trans>
              )
            ) : isSwap ? (
              <Trans>
                The offer to fulfill the swap has been created successfully. It
                will now be executed on Dexie and imported on the offers page.
              </Trans>
            ) : uploadedToMarketplaces ? (
              <Trans>
                Your offer has been created and imported successfully and
                uploaded to the selected marketplaces. You will now be
                redirected to the offers page where you can view its details.
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
          {isProcessing || isUploading ? (
            <Button variant='outline' onClick={handleCancel}>
              <Trans>Cancel</Trans>
            </Button>
          ) : (
            <Button onClick={() => handleDone()}>
              <Trans>Done</Trans>
            </Button>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

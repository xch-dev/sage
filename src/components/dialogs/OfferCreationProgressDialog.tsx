import { commands, NetworkKind } from '@/bindings';
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
import { useOfferProcessor } from '@/hooks/useOfferProcessor';
import { marketplaces } from '@/lib/marketplaces';
import { OfferState } from '@/state';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { LoaderCircleIcon } from 'lucide-react';
import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';

const delay = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

interface OfferCreationProgressDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  offerState: OfferState;
  splitNftOffers: boolean;
  enabledMarketplaces: { [key: string]: boolean };
  clearOfferState: () => void;
}

export function OfferCreationProgressDialog({
  open,
  onOpenChange,
  offerState,
  splitNftOffers,
  enabledMarketplaces,
  clearOfferState,
}: OfferCreationProgressDialogProps) {
  const { addError } = useErrors();
  const navigate = useNavigate();
  const [network, setNetwork] = useState<NetworkKind | null>(null);
  const [isUploading, setIsUploading] = useState(false);
  const [hasStartedProcessing, setHasStartedProcessing] = useState(false);
  const [isCanceling, setIsCanceling] = useState(false);
  const [uploadsCompleted, setUploadsCompleted] = useState(false);
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

  useEffect(() => {
    commands.getNetwork({}).then((data) => setNetwork(data.kind));
  }, []);

  // Handle uploads when offers are created
  useEffect(() => {
    if (createdOffers.length > 0 && network && !isProcessing && !isCanceling) {
      let isMounted = true;

      const uploadToMarketplaces = async () => {
        const enabledMarketplaceConfigs = marketplaces.filter(
          (marketplace) => enabledMarketplaces[marketplace.id],
        );

        if (enabledMarketplaceConfigs.length === 0) {
          setUploadsCompleted(true);
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
              await marketplace.uploadToMarketplace(
                individualOffer,
                network === 'testnet',
              );
              if (offerIndex < createdOffers.length - 1) {
                // rate limit
                await delay(500);
              }
            } catch (error) {
              if (isMounted) {
                addError({
                  kind: 'upload',
                  reason: t`Failed to auto-upload offer ${offerIndex + 1} to ${marketplace.name}. Stopping.: ${error}`,
                });
                // typically if one fails the rest will fail too
                break;
              }
            }
          }
        }

        if (isMounted && !isCanceling) {
          setIsUploading(false);
          setUploadsCompleted(true);
        }
      };

      uploadToMarketplaces();

      return () => {
        isMounted = false;
      };
    }
  }, [
    createdOffers,
    network,
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
      setUploadsCompleted(false);
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
      await cancelProcessing();
    }
    clearProcessedOffers();
    setIsUploading(false);
    setUploadsCompleted(false);
    onOpenChange(false);
  };

  const handleDone = () => {
    clearProcessedOffers();
    clearOfferState();
    navigate('/offers', { replace: true });
  };

  const getProgressMessage = () => {
    if (isProcessing || isUploading) {
      if (currentStep === 'creating') {
        return (
          <Trans>
            Creating offer {currentOfferIndex + 1} of {totalOffers}...
          </Trans>
        );
      } else if (currentStep === 'uploading') {
        const enabledMarketplaceConfigs = marketplaces.filter(
          (marketplace) => enabledMarketplaces[marketplace.id],
        );
        const currentMarketplace =
          enabledMarketplaceConfigs[currentMarketplaceIndex];
        return (
          <Trans>
            Uploading offer {currentOfferIndex + 1} of {totalOffers} to{' '}
            {currentMarketplace.name}...
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
                <p>
                  <Trans>
                    Please wait while{' '}
                    {splitNftOffers ? 'your offers are' : 'your offer is'} being
                    {currentStep === 'creating' ? ' created' : ' uploaded'}
                    {currentStep === 'creating' &&
                    Object.values(enabledMarketplaces).some(Boolean)
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
                successfully
                {Object.values(enabledMarketplaces).some(Boolean)
                  ? ' and uploaded to the selected marketplaces'
                  : ''}
                . You will now be redirected to the offers page where you can
                view the details of each offer.
              </Trans>
            ) : (
              <Trans>
                Your offer has been created and imported successfully
                {Object.values(enabledMarketplaces).some(Boolean)
                  ? ' and uploaded to the selected marketplaces'
                  : ''}
                . You will now be redirected to the offers page where you can
                view its details.
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
            <Button onClick={handleDone}>
              <Trans>Done</Trans>
            </Button>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

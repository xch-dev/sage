import { OfferRecord } from '@/bindings';
import { MakeOfferConfirmationDialog } from '@/components/dialogs/MakeOfferConfirmationDialog';
import { OfferCreationProgressDialog } from '@/components/dialogs/OfferCreationProgressDialog';
import { offerStateFromRecord } from '@/lib/offers';
import { useMemo, useRef, useState } from 'react';

interface DuplicateOfferDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  record: OfferRecord;
  onDone: () => void;
}

// Creates copies of a saved offer through the same confirmation and progress
// dialogs used by the make offer page.
export function DuplicateOfferDialog({
  open,
  onOpenChange,
  record,
  onDone,
}: DuplicateOfferDialogProps) {
  const offerState = useMemo(() => offerStateFromRecord(record), [record]);
  const [copies, setCopies] = useState(1);
  const [enabledMarketplaces, setEnabledMarketplaces] = useState<
    Record<string, boolean>
  >({});
  const [isProgressOpen, setIsProgressOpen] = useState(false);
  // The confirmation dialog closes itself right after confirming, which must
  // not reset the choices the progress dialog is about to use.
  const confirmedRef = useRef(false);

  const reset = () => {
    setCopies(1);
    setEnabledMarketplaces({});
  };

  // Runs whether creation finished or failed partway, since any offers that
  // were made are already in the wallet.
  const finish = () => {
    setIsProgressOpen(false);
    reset();
    onDone();
  };

  return (
    <>
      <MakeOfferConfirmationDialog
        open={open}
        onOpenChange={(isOpen) => {
          onOpenChange(isOpen);
          if (!isOpen && !confirmedRef.current) reset();
          confirmedRef.current = false;
        }}
        onConfirm={() => {
          confirmedRef.current = true;
          setIsProgressOpen(true);
        }}
        offerState={offerState}
        splitNftOffers={false}
        fee={offerState.fee}
        enabledMarketplaces={enabledMarketplaces}
        setEnabledMarketplaces={setEnabledMarketplaces}
        copies={copies}
        onCopiesChange={setCopies}
      />

      <OfferCreationProgressDialog
        open={isProgressOpen}
        onOpenChange={(isOpen) => {
          if (!isOpen) finish();
        }}
        offerState={offerState}
        splitNftOffers={false}
        enabledMarketplaces={enabledMarketplaces}
        copies={copies}
        clearOfferState={finish}
      />
    </>
  );
}

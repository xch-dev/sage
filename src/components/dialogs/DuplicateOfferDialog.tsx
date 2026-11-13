import { OfferRecord } from '@/bindings';
import { MakeOfferConfirmationDialog } from '@/components/dialogs/MakeOfferConfirmationDialog';
import { OfferCreationProgressDialog } from '@/components/dialogs/OfferCreationProgressDialog';
import { useDefaultOfferExpiry } from '@/hooks/useDefaultOfferExpiry';
import { offerStateFromRecord } from '@/lib/offers';
import { useMemo, useRef, useState } from 'react';

interface DuplicateOfferDialogProps {
  record: OfferRecord;
  // Called once the flow ends, whether it was cancelled, failed or finished.
  // Any offers created before a failure are already saved in the wallet.
  onClose: () => void;
}

// Creates copies of a saved offer through the same confirmation and progress
// dialogs used by the make offer page. Mount it only while duplicating, and
// outside the offer list, so that a list refresh can't unmount it mid-run.
export function DuplicateOfferDialog({
  record,
  onClose,
}: DuplicateOfferDialogProps) {
  const { expiry } = useDefaultOfferExpiry();
  // Keyed on primitives, since the expiry object is rebuilt every render.
  const offerState = useMemo(
    () =>
      offerStateFromRecord(
        record,
        expiry.enabled
          ? { days: expiry.days, hours: expiry.hours, minutes: expiry.minutes }
          : null,
      ),
    [record, expiry.enabled, expiry.days, expiry.hours, expiry.minutes],
  );
  const [copies, setCopies] = useState(1);
  const [enabledMarketplaces, setEnabledMarketplaces] = useState<
    Record<string, boolean>
  >({});
  const [step, setStep] = useState<'confirm' | 'progress'>('confirm');
  // The confirmation dialog closes itself right after confirming, which
  // must not end the flow.
  const confirmedRef = useRef(false);

  return (
    <>
      <MakeOfferConfirmationDialog
        open={step === 'confirm'}
        onOpenChange={(isOpen) => {
          if (!isOpen && !confirmedRef.current) onClose();
        }}
        onConfirm={() => {
          confirmedRef.current = true;
          setStep('progress');
        }}
        offerState={offerState}
        splitNftOffers={false}
        preSplitFee={null}
        fee={offerState.fee}
        enabledMarketplaces={enabledMarketplaces}
        setEnabledMarketplaces={setEnabledMarketplaces}
        copies={copies}
        onCopiesChange={setCopies}
      />

      <OfferCreationProgressDialog
        open={step === 'progress'}
        onOpenChange={(isOpen) => {
          if (!isOpen) onClose();
        }}
        offerState={offerState}
        splitNftOffers={false}
        preSplitFee={null}
        enabledMarketplaces={enabledMarketplaces}
        copies={copies}
        clearOfferState={onClose}
      />
    </>
  );
}

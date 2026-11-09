import { OfferRecord } from '@/bindings';
import { DeleteOfferDialog } from '@/components/dialogs/DeleteOfferDialog';
import { CustomError } from '@/contexts/ErrorContext';
import { useWallet } from '@/contexts/WalletContext';
import { useErrors } from '@/hooks/useErrors';
import { deleteOffers, fetchOfferRecords } from '@/lib/offers';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { CircleOff, TrashIcon } from 'lucide-react';
import { useState } from 'react';
import { toast } from 'react-toastify';
import { CancelOffersFlow } from './dialogs/CancelOffersFlow';
import { MultiSelectActionBar } from './MultiSelectActionBar';
import { DropdownMenuItem, DropdownMenuSeparator } from './ui/dropdown-menu';

export interface OffersMultiSelectActionsProps {
  selected: string[];
  onConfirm: () => void;
  onSelectAll?: () => void;
  onClearSelection?: () => void;
}

export function OffersMultiSelectActions({
  selected,
  onConfirm,
  onSelectAll,
  onClearSelection,
}: OffersMultiSelectActionsProps) {
  const { isTransactionDisabled } = useWallet();
  const { addError } = useErrors();

  const [isDeleteOpen, setIsDeleteOpen] = useState(false);
  const [isCancelOpen, setIsCancelOpen] = useState(false);
  const [isLoadingCancel, setIsLoadingCancel] = useState(false);

  // Populated only once the user opens the cancel flow, with a fresh,
  // filtered-to-active read of the selected offers (see openCancelFlow).
  // Selections can span pages, and an offer can be deleted or completed after
  // it was selected, so we can't trust records fetched earlier for the
  // selection.
  const [cancelOffers, setCancelOffers] = useState<OfferRecord[]>([]);

  const selectedCount = selected.length;
  const cancelOffersCount = cancelOffers.length;

  const openCancelFlow = async () => {
    setIsLoadingCancel(true);

    try {
      const records = await fetchOfferRecords(selected, (error) =>
        addError(error as CustomError),
      );
      const activeOffers = records.filter((offer) => offer.status === 'active');

      if (activeOffers.length === 0) {
        toast.info(t`None of the selected offers are active`);
        return;
      }

      setCancelOffers(activeOffers);
      setIsCancelOpen(true);
    } finally {
      setIsLoadingCancel(false);
    }
  };

  return (
    <>
      <MultiSelectActionBar
        selectedCount={selectedCount}
        regionAriaLabel={t`Selected offers actions`}
        actionsAriaLabel={t`Actions for ${selectedCount} selected offers`}
        selectAllAriaLabel={t`Select all offers on this page`}
        onSelectAll={onSelectAll}
        onClearSelection={onClearSelection}
      >
        <DropdownMenuItem
          className='cursor-pointer'
          disabled={
            isTransactionDisabled || selectedCount === 0 || isLoadingCancel
          }
          onClick={(e) => {
            e.stopPropagation();
            openCancelFlow();
          }}
          aria-label={t`Cancel ${selectedCount} selected offers`}
        >
          <CircleOff className='mr-2 h-4 w-4' aria-hidden='true' />
          <span>
            <Trans>Cancel Offers</Trans>
          </span>
        </DropdownMenuItem>

        <DropdownMenuSeparator />

        <DropdownMenuItem
          className='cursor-pointer'
          onClick={(e) => {
            e.stopPropagation();
            setIsDeleteOpen(true);
          }}
          aria-label={t`Delete ${selectedCount} selected offers`}
        >
          <TrashIcon className='mr-2 h-4 w-4' aria-hidden='true' />
          <span>
            <Trans>Delete</Trans>
          </span>
        </DropdownMenuItem>
      </MultiSelectActionBar>

      <CancelOffersFlow
        open={isCancelOpen}
        onOpenChange={setIsCancelOpen}
        offers={cancelOffers}
        onConfirm={onConfirm}
        title={<Trans>Cancel selected offers?</Trans>}
        description={
          <Trans>
            This will cancel the {cancelOffersCount} active offers in your
            selection on-chain with a transaction, preventing them from being
            taken even if someone has the original offer files.
          </Trans>
        }
      />

      <DeleteOfferDialog
        open={isDeleteOpen}
        onOpenChange={setIsDeleteOpen}
        offerCount={selectedCount}
        onDelete={() => {
          deleteOffers(selected)
            .then(onConfirm)
            .catch((error) =>
              addError({
                kind: 'internal',
                reason: `Failed to delete offers: ${error}`,
              }),
            )
            .finally(() => setIsDeleteOpen(false));
        }}
      />
    </>
  );
}

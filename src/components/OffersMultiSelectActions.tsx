import { OfferRecord } from '@/bindings';
import { DeleteOfferDialog } from '@/components/dialogs/DeleteOfferDialog';
import { useWallet } from '@/contexts/WalletContext';
import { useErrors } from '@/hooks/useErrors';
import { deleteOffers } from '@/lib/offers';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { CircleOff, TrashIcon } from 'lucide-react';
import { useState } from 'react';
import { CancelOffersFlow } from './dialogs/CancelOffersFlow';
import { MultiSelectActionBar } from './MultiSelectActionBar';
import { DropdownMenuItem, DropdownMenuSeparator } from './ui/dropdown-menu';

export interface OffersMultiSelectActionsProps {
  selected: string[];
  offers: OfferRecord[];
  onConfirm: () => void;
  onSelectAll?: () => void;
  onClearSelection?: () => void;
}

export function OffersMultiSelectActions({
  selected,
  offers,
  onConfirm,
  onSelectAll,
  onClearSelection,
}: OffersMultiSelectActionsProps) {
  const { isTransactionDisabled } = useWallet();
  const { addError } = useErrors();

  const [isDeleteOpen, setIsDeleteOpen] = useState(false);
  const [isCancelOpen, setIsCancelOpen] = useState(false);

  const selectedCount = selected.length;
  const selectedOffers = offers.filter((offer) =>
    selected.includes(offer.offer_id),
  );
  const activeSelectedOffers = selectedOffers.filter(
    (offer) => offer.status === 'active',
  );
  const activeSelectedCount = activeSelectedOffers.length;

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
          disabled={isTransactionDisabled || activeSelectedCount === 0}
          onClick={(e) => {
            e.stopPropagation();
            setIsCancelOpen(true);
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
        offers={activeSelectedOffers}
        onConfirm={onConfirm}
        title={<Trans>Cancel selected offers?</Trans>}
        description={
          <Trans>
            This will cancel the {activeSelectedCount} active offers in your
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

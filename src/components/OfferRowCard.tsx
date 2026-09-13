import { OfferRecord } from '@/bindings';
import { CancelOffersFlow } from '@/components/dialogs/CancelOffersFlow';
import { DeleteOfferDialog } from '@/components/dialogs/DeleteOfferDialog';
import { OfferSummaryCard } from '@/components/OfferSummaryCard';
import { SelectableCard, SelectionState } from '@/components/SelectableCard';
import { Button } from '@/components/ui/button';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { useWallet } from '@/contexts/WalletContext';
import { useErrors } from '@/hooks/useErrors';
import { deleteOffers } from '@/lib/offers';
import { Trans } from '@lingui/react/macro';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import {
  CircleOff,
  CopyIcon,
  MoreVertical,
  Tags,
  TrashIcon,
} from 'lucide-react';
import { useState } from 'react';
import { useNavigate } from 'react-router-dom';

interface OfferRowCardProps {
  record: OfferRecord;
  refresh: () => void;
  selectionState?: SelectionState;
}

export function OfferRowCard({
  record,
  refresh,
  selectionState = null,
}: OfferRowCardProps) {
  const navigate = useNavigate();
  const { isTransactionDisabled } = useWallet();
  const { addError } = useErrors();
  const [isDeleteOpen, setIsDeleteOpen] = useState(false);
  const [isCancelOpen, setIsCancelOpen] = useState(false);

  return (
    <>
      <SelectableCard
        selectionState={selectionState}
        onOpen={() => navigate(`/offers/view_saved/${record.offer_id.trim()}`)}
      >
        <OfferSummaryCard
          record={record}
          selectionState={selectionState}
          content={
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <Button
                  variant='ghost'
                  size='icon'
                  className='-mr-1.5 flex-shrink-0'
                  onClick={(e) => e.stopPropagation()}
                >
                  <MoreVertical className='h-5 w-5' aria-hidden='true' />
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent align='end'>
                <DropdownMenuGroup>
                  <DropdownMenuItem
                    className='cursor-pointer'
                    onClick={(e) => {
                      e.stopPropagation();
                      writeText(record.offer);
                    }}
                  >
                    <CopyIcon className='mr-2 h-4 w-4' aria-hidden='true' />
                    <span>
                      <Trans>Copy</Trans>
                    </span>
                  </DropdownMenuItem>

                  <DropdownMenuItem
                    className='cursor-pointer'
                    onClick={(e) => {
                      e.stopPropagation();
                      setIsDeleteOpen(true);
                    }}
                  >
                    <TrashIcon className='mr-2 h-4 w-4' aria-hidden='true' />
                    <span>
                      <Trans>Delete</Trans>
                    </span>
                  </DropdownMenuItem>

                  <DropdownMenuItem
                    className='cursor-pointer'
                    onClick={(e) => {
                      e.stopPropagation();
                      setIsCancelOpen(true);
                    }}
                    disabled={
                      record.status !== 'active' || isTransactionDisabled
                    }
                  >
                    <CircleOff className='mr-2 h-4 w-4' aria-hidden='true' />
                    <Trans>Cancel</Trans>
                  </DropdownMenuItem>

                  <DropdownMenuSeparator />

                  <DropdownMenuItem
                    className='cursor-pointer'
                    onClick={(e) => {
                      e.stopPropagation();
                      writeText(record.offer_id);
                    }}
                  >
                    <Tags className='mr-2 h-4 w-4' />
                    <span>
                      <Trans>Copy ID</Trans>
                    </span>
                  </DropdownMenuItem>
                </DropdownMenuGroup>
              </DropdownMenuContent>
            </DropdownMenu>
          }
        />
      </SelectableCard>

      <DeleteOfferDialog
        open={isDeleteOpen}
        onOpenChange={setIsDeleteOpen}
        offerCount={1}
        onDelete={() => {
          deleteOffers([record.offer_id])
            .then(refresh)
            .catch(addError)
            .finally(() => setIsDeleteOpen(false));
        }}
      />

      <CancelOffersFlow
        open={isCancelOpen}
        onOpenChange={setIsCancelOpen}
        offers={[record]}
        onConfirm={refresh}
      />
    </>
  );
}

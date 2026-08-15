import { commands, OfferRecord, TransactionResponse } from '@/bindings';
import ConfirmationDialog from '@/components/ConfirmationDialog';
import { CancelOfferDialog } from '@/components/dialogs/CancelOfferDialog';
import { DeleteOfferDialog } from '@/components/dialogs/DeleteOfferDialog';
import { DuplicateOfferDialog } from '@/components/dialogs/DuplicateOfferDialog';
import { OfferSummaryCard } from '@/components/OfferSummaryCard';
import { Button } from '@/components/ui/button';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { useErrors } from '@/hooks/useErrors';
import { amount } from '@/lib/formTypes';
import { toMojos } from '@/lib/utils';
import { useWalletState } from '@/state';
import { zodResolver } from '@hookform/resolvers/zod';
import { t } from '@lingui/core/macro';
import { Trans } from '@lingui/react/macro';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import BigNumber from 'bignumber.js';
import {
  CircleOff,
  CopyIcon,
  CopyPlus,
  MoreVertical,
  Tags,
  TrashIcon,
} from 'lucide-react';
import { useState } from 'react';
import { useForm } from 'react-hook-form';
import { Link } from 'react-router-dom';
import { z } from 'zod';
import { CancelOfferConfirmation } from './confirmations/CancelOfferConfirmation';

interface OfferRowCardProps {
  record: OfferRecord;
  refresh: () => void;
}

export function OfferRowCard({ record, refresh }: OfferRowCardProps) {
  const walletState = useWalletState();
  const { addError } = useErrors();
  const [isDeleteOpen, setIsDeleteOpen] = useState(false);
  const [isCancelOpen, setIsCancelOpen] = useState(false);
  const [isDuplicateOpen, setIsDuplicateOpen] = useState(false);

  const isTokenOnlyOffer = record.summary.maker.every(
    (a) => a.asset.kind === 'token',
  );

  const cancelSchema = z.object({
    fee: amount(walletState.sync.unit.precision).refine(
      (amount) =>
        BigNumber(walletState.sync.selectable_balance).gte(amount || 0),
      t`Not enough funds to cover the fee`,
    ),
  });

  const cancelForm = useForm<z.infer<typeof cancelSchema>>({
    resolver: zodResolver(cancelSchema),
  });

  const [response, setResponse] = useState<TransactionResponse | null>(null);

  const cancelHandler = (values: z.infer<typeof cancelSchema>) => {
    const fee = toMojos(values.fee, walletState.sync.unit.precision);

    commands
      .cancelOffer({
        offer_id: record.offer_id,
        fee,
      })
      .then((result) => {
        setResponse(result);
      })
      .catch(addError)
      .finally(() => setIsCancelOpen(false));
  };

  return (
    <>
      <Link to={`/offers/view_saved/${record.offer_id.trim()}`}>
        <OfferSummaryCard
          record={record}
          content={
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <Button
                  variant='ghost'
                  size='icon'
                  className='-mr-1.5 flex-shrink-0'
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
                    disabled={record.status !== 'active'}
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

                  <DropdownMenuSeparator />

                  <DropdownMenuItem
                    className='cursor-pointer'
                    onClick={(e) => {
                      e.stopPropagation();
                      setIsDuplicateOpen(true);
                    }}
                    disabled={!isTokenOnlyOffer}
                  >
                    <CopyPlus className='mr-2 h-4 w-4' aria-hidden='true' />
                    <span>
                      <Trans>Duplicate</Trans>
                    </span>
                  </DropdownMenuItem>
                </DropdownMenuGroup>
              </DropdownMenuContent>
            </DropdownMenu>
          }
        />
      </Link>

      <DeleteOfferDialog
        open={isDeleteOpen}
        onOpenChange={setIsDeleteOpen}
        offerCount={1}
        onDelete={() => {
          commands
            .deleteOffer({ offer_id: record.offer_id })
            .then(() => refresh())
            .catch(addError)
            .finally(() => setIsDeleteOpen(false));
        }}
      />

      <CancelOfferDialog
        open={isCancelOpen}
        onOpenChange={setIsCancelOpen}
        form={cancelForm}
        onSubmit={cancelHandler}
      />

      <ConfirmationDialog
        response={response}
        showRecipientDetails={false}
        close={() => setResponse(null)}
        onConfirm={refresh}
        additionalData={{
          title: t`Cancel Offer`,
          content: response && <CancelOfferConfirmation offers={[record]} />,
        }}
      />

      <DuplicateOfferDialog
        open={isDuplicateOpen}
        onOpenChange={setIsDuplicateOpen}
        record={record}
        onDone={refresh}
      />
    </>
  );
}

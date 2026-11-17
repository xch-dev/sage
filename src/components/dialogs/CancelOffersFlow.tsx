import { commands, OfferRecord, TransactionResponse } from '@/bindings';
import ConfirmationDialog from '@/components/ConfirmationDialog';
import { CancelOfferConfirmation } from '@/components/confirmations/CancelOfferConfirmation';
import { useErrors } from '@/hooks/useErrors';
import { feeParams, feeWithinBalance, isAutoFee } from '@/lib/fees';
import { useWalletState } from '@/state';
import { zodResolver } from '@hookform/resolvers/zod';
import { t } from '@lingui/core/macro';
import { ReactNode, useState } from 'react';
import { useForm } from 'react-hook-form';
import { z } from 'zod';
import { CancelOfferDialog } from './CancelOfferDialog';

export interface CancelOffersFlowProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  offers: OfferRecord[];
  onConfirm: () => void;
  title?: ReactNode;
  description?: ReactNode;
}

export function CancelOffersFlow({
  open,
  onOpenChange,
  offers,
  onConfirm,
  title,
  description,
}: CancelOffersFlowProps) {
  const walletState = useWalletState();
  const { addError } = useErrors();
  const [response, setResponse] = useState<TransactionResponse | null>(null);
  const [fee, setFee] = useState('');
  const [pending, setPending] = useState(false);

  const schema = z.object({
    fee: feeWithinBalance(
      walletState.sync.unit.precision,
      walletState.sync.selectable_balance,
    ),
  });

  const form = useForm<z.infer<typeof schema>>({
    resolver: zodResolver(schema),
  });

  const submit = (values: z.infer<typeof schema>) => {
    setPending(true);
    setFee(
      isAutoFee(values.fee)
        ? t`Automatic`
        : `${values.fee} ${walletState.sync.unit.ticker}`,
    );

    feeParams(values.fee, walletState.sync.unit.precision)
      .then((fees) =>
        commands.cancelOffers({
          offer_ids: offers.map((offer) => offer.offer_id),
          ...fees,
        }),
      )
      .then(setResponse)
      .catch(addError)
      .finally(() => {
        setPending(false);
        onOpenChange(false);
      });
  };

  return (
    <>
      <CancelOfferDialog
        open={open}
        onOpenChange={onOpenChange}
        form={form}
        onSubmit={submit}
        title={title}
        description={description}
        feeLabel={offers.length > 1 ? t`Network Fee (per offer)` : undefined}
        pending={pending}
      />

      <ConfirmationDialog
        response={response}
        showRecipientDetails={false}
        close={() => {
          setResponse(null);
          setFee('');
        }}
        onConfirm={onConfirm}
        additionalData={{
          title: offers.length > 1 ? t`Cancel Offers` : t`Cancel Offer`,
          content: response && (
            <CancelOfferConfirmation offers={offers} fee={fee} />
          ),
        }}
      />
    </>
  );
}
